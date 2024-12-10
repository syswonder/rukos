/* Copyright (c) [2024] [Syswonder Community]
*   [Ruxos] is licensed under Mulan PSL v2.
*   You can use this software according to the terms and conditions of the Mulan PSL v2.
*   You may obtain a copy of Mulan PSL v2 at:
*               http://license.coscl.org.cn/MulanPSL2
*   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
*   See the Mulan PSL v2 for more details.
*/

use core::alloc::Layout;
use core::ptr::NonNull; 
use axalloc::global_allocator;
use memory_addr::{PhysAddr, VirtAddr};
use ruxconfig::IVC_ZONES;
use crate::mem::{direct_virt_to_phys, phys_to_virt};
#[cfg(all(target_arch = "aarch64", feature = "ivc"))] 
use crate::platform::ivc::ivc_hvc_call;

pub const CONFIG_MAX_IVC_CONFIGS: usize = 0x2;
pub const HVISOR_HC_IVC_INFO: usize = 0x5;
pub const HVISOR_HC_IVC_INFO_ALIGN: usize = 0x8;
pub const HVISOR_HC_IVC_INFO_SIZE: usize = 56;
pub const __PA: usize = 0xffff_0000_0000_0000;

#[repr(C)]
#[derive(Debug)]
struct IvCInfo {
    /// The number of IVC shared memory 
    len: u64,   
    /// Control Table IPA
    ivc_ct_ipas: [u64; IVC_ZONES],
    /// Share memory IPA
    ivc_shmem_ipas: [u64; IVC_ZONES], 
    /// IVC id; the ivc id of zones that communicate with each other have to be the same
    ivc_ids: [u32; IVC_ZONES], 
    /// irq number
    ivc_irqs: [u32; IVC_ZONES], 
}

#[repr(C)]
#[derive(Debug)]
struct ControlTable {
    ivc_id: u32,
    max_peers: u32,
    rw_sec_size: u32,
    out_sec_size: u32,
    peer_id: u32,
    ipi_invoke: u32,
}

/// This module provides a way to establish a communication channel with a hypervisor (hvisor) 
/// for virtual machine (VM) communication using shared memory. Each VM can have two communication 
/// regions, and the region to be used for communication is specified during the connection setup.
///
/// The communication is handled through the following steps:
/// 
/// 1. **Connection Setup**: The `connect()` function allocates memory for communication structures
///    and invokes a hypervisor call (`hvc`) to retrieve necessary information about the IVC 
///    (Inter-VM Communication) and control tables. The specific communication region to be used 
///    is determined by the parameter passed to `connect()`. This process sets up the shared memory 
///    and prepares the communication channel.
/// 2. **Message Sending**: The `send_message()` function writes a message to the shared memory area 
///    specified by the hypervisor. The message is written to a predefined memory location, and 
///    the control table is updated to notify the target VM of the message.
/// 3. **Connection Teardown**: The `close()` function frees the allocated memory and closes the 
///    communication channel, cleaning up resources to prevent memory leaks.
///
/// The module relies on a `GlobalAllocator` for memory management and uses raw pointers and unsafe 
/// Rust operations to interact with memory addresses provided by the hypervisor. It is critical 
/// that the communication sequence follows the correct order: connect -> send_message -> close.
///
/// # Example
/// ```
/// let mut conn = Connection::new();
/// if let Err(e) = conn.connect(0) {    // Choose the first communication region (0)
///     info!("Error connecting: {}", e);
///     return;
/// }
/// if let Err(e) = conn.send_message("Hello from zone1 ruxos!") {
///     error!("Error sending message: {}", e);
/// }
/// if let Err(e) = conn.close() {
///     error!("Error closing connection: {}", e);
/// }
/// ```

pub fn ivc_example() {
    let mut conn = Connection::new();

    // Establish the connection
    if let Err(e) = conn.connect(0) {
        info!("Error connecting: {}", e);
        return;
    }

    // Send the message
    if let Err(e) = conn.send_message("Hello from zone1 ruxos! ") {
        info!("Error sending message: {}", e);
    }

    // Close the connection
    if let Err(e) = conn.close() {
        info!("Error closing connection: {}", e);
    }
}

pub struct Connection { 
    ivc_info: Option<IvCInfo>,
    control_table: Option<NonNull<ControlTable>>,
}

impl Connection {
    pub fn new() -> Self {
        debug!("Connection created.");
        Connection {
            ivc_info: None,
            control_table: None,
        }
    }

    pub fn connect(&mut self, communication_zone: usize) -> Result<(), &'static str> {
        let alloc_size = HVISOR_HC_IVC_INFO_SIZE;
        let align = HVISOR_HC_IVC_INFO_ALIGN;
        let layout = Layout::from_size_align(alloc_size, align).unwrap();
        
        let ptr = global_allocator().alloc(layout).expect("Memory allocation failed!");
        
        let vpa_ivcinfo = VirtAddr::from(ptr.as_ptr() as usize);
        // convert the virtual address to physical address, to use hvc on physical address
        let pa_ivcinfo: PhysAddr = direct_virt_to_phys(vpa_ivcinfo);
        debug!("The memory address of the IVC Info: VA: 0x{:x}, IPA: 0x{:x}", vpa_ivcinfo.as_usize(), pa_ivcinfo.as_usize());

        ivc_hvc_call(HVISOR_HC_IVC_INFO as u32, pa_ivcinfo.as_usize(), HVISOR_HC_IVC_INFO_SIZE);
        debug!("ivc_hvc_call finished.");

        let ivc_info_ptr = ptr.as_ptr() as *const IvCInfo;
        let ivc_info: IvCInfo = unsafe { ivc_info_ptr.read() };
        self.ivc_info = Some(ivc_info);

        global_allocator().dealloc(ptr, layout);

        let ivc_info = self.ivc_info.as_ref().unwrap();
        let pa_control_table = PhysAddr::from(ivc_info.ivc_ct_ipas[communication_zone] as usize);
        // convert the physical address to virtual address to use it in the kernel
        let vpa_control_table: VirtAddr = phys_to_virt(pa_control_table);
        self.control_table = NonNull::new(vpa_control_table.as_ptr() as *mut ControlTable);

        info!("IVC Connection established.");
        Ok(())
    }

    pub fn send_message(&mut self, message: &str) -> Result<(), &'static str> {
        let ivc_info = self.ivc_info.as_ref().ok_or("Not connected")?;
        let mut control_table_ptr = self.control_table.ok_or("Not connected")?; 
        
        // Safely get an immutable reference to the ControlTable for reading out_sec_size
        let control_table = unsafe { control_table_ptr.as_ref() };
    
        // Suppose we are zone1, we are to send message to zone0.
        // Therefore use the out_sec_size field of the ControlTable struct (typically 0x1000).
        let offset = control_table.out_sec_size as u64;
        let vpa_share_memory_zone1 = phys_to_virt(PhysAddr::from((ivc_info.ivc_shmem_ipas[0] + offset) as usize));
    
        write_to_address(vpa_share_memory_zone1, message)?;
        info!("Message written to shared memory: {}", message);
    
        // Get a mutable reference to ControlTable to modify ipi_invoke
        let control_table = unsafe { control_table_ptr.as_mut() };
        debug!("Ipi_invoke reset to inform Zone0 linux.");
        control_table.ipi_invoke = 0x0;
    
        Ok(())
    }

    pub fn close(&mut self) -> Result<(), &'static str> {
        if self.ivc_info.is_none() {
            return Err("Not connected");
        }
        self.ivc_info = None;
        self.control_table = None;
        info!("IVC Connection closed.");
        Ok(())
    }
}

/// Writes the given data to the specified virtual address.
fn write_to_address(addr: VirtAddr, data: &str) -> Result<(), &'static str> {
    unsafe {
        let ptr = addr.as_usize() as *mut u8;
        ptr.copy_from_nonoverlapping(data.as_ptr(), data.len());
        // Add a null terminator to the end of the string
        *ptr.add(data.len()) = 0;
    }
    Ok(())
}
