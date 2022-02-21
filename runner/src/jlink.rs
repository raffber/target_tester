use std::ffi::{c_void, CStr, CString};
use std::net::SocketAddr;
use std::os::raw::{c_char, c_int, c_uint};
use crate::bindings::JLINK_API;
use crate::jlink_sys::JLINKARM_SPEED_INVALID;
use crate::{Interface, JLINKARM_SPEED_ADAPTIVE, JLINKARM_SPEED_AUTO, JLINKARM_TIF_JTAG, JLINKARM_TIF_SWD, Speed};

pub fn open(addr: Option<SocketAddr>) -> Result<(), String> {
    if let Some(addr) = addr {
        let ip = CString::new(addr.ip().to_string()).unwrap();
        unsafe {
            let result = JLINK_API.JLINKARM_SelectIP(ip.as_ptr(), addr.port() as i32);
            if result == 1 {
                return Err(format!("Could not connect to emulator on {}", addr));
            }
        }
    } else {
        unsafe {
            let result = JLINK_API.JLINKARM_SelectUSB(0);
            if result != 0 {
                return Err("Could not connect to default emulator!".to_string());
            }
        }
    }
    let c_str = unsafe {
        let data = JLINK_API.JLINK_Open();
        if !data.is_null() {
            Some(CStr::from_ptr(data))
        } else {
            None
        }
    };
    if let Some(err) = c_str {
        let err = err.to_str().unwrap().to_string();
        return Err(err);
    }
    Ok(())
}

pub fn exec_command(cmd: &str) -> Result<i32, String> {
    let cmd = CString::new(cmd).unwrap();
    const OUT_BUF_SIZE: usize = 256;
    let mut out_buf = [0 as c_char; OUT_BUF_SIZE];
    let ret = unsafe {
        let ret = JLINK_API.JLINKARM_ExecCommand(cmd.as_ptr(), &mut out_buf as *mut c_char, OUT_BUF_SIZE as c_int);
        if out_buf[0] != 0 {
            let str = CString::from_raw(&mut out_buf as *mut c_char);
            return Err(str.to_str().unwrap().to_string());
        }
        ret
    };

    Ok(ret as i32)
}

pub fn connect() -> Result<(), String> {
    let result = unsafe {
        JLINK_API.JLINKARM_Connect()
    };
    if result < 0 {
        return Err(format!("Connection failed with error code `{}`", result));
    }
    Ok(())
}

pub fn set_tif(tif: Interface) -> Result<(), String> {
    let tif = match tif {
        Interface::JTAG => JLINKARM_TIF_JTAG,
        Interface::SWD => JLINKARM_TIF_SWD,
    };
    unsafe {
        let ret = JLINK_API.JLINKARM_TIF_Select(tif as c_int);
        if ret != 0 {
            return Err(format!("Cannot select given interface."));
        }
    }
    Ok(())
}

pub fn set_speed(speed: Speed) -> Result<(), String> {
    let speed = match speed {
        Speed::Auto => {
            JLINKARM_SPEED_AUTO
        }
        Speed::Adaptive => {
            JLINKARM_SPEED_ADAPTIVE
        }
        Speed::KHz(x) => {
            let x = x as u32;
            if x > JLINKARM_SPEED_INVALID || x == 0 {
                return Err(format!("Invalid speed given."));
            }
            x
        }
    };
    unsafe {
        JLINK_API.JLINKARM_SetSpeed(speed);
    }
    Ok(())
}

pub fn reset_device() -> Result<(), String> {
    let status = unsafe {
        JLINK_API.JLINKARM_Reset()
    };
    if status < 0 {
        return Err(format!("Cannot reset target."));
    }
    Ok(())
}

pub fn download(addr: u64, data: &[u8]) -> Result<(), String> {
    unsafe {
        JLINK_API.JLINK_BeginDownload(0);
        let stuff = JLINK_API.JLINK_WriteMem(addr as u32, data.len() as u32, data.as_ptr() as *const c_void);
        println!("JLINK_WriteMem = {}", stuff);
        let stuff = JLINK_API.JLINK_EndDownload();
        println!("JLINK_EndDownload = {}", stuff);
    }
    Ok(())
}

pub fn set_breakpoint(idx: u32, addr: u64) -> Result<(), String> {
    let stuff = unsafe {
        JLINK_API.JLINKARM_SetBP(idx, addr as u32)
    };
    println!("JLINKARM_SetBP returned {}", stuff);
    Ok(())
}

pub fn clear_breakpoint(idx: u32) -> Result<(), String> {
    let stuff = unsafe {
        JLINK_API.JLINKARM_ClrBP(idx)
    };
    println!("JLINKARM_ClrBP returned {}", stuff);
    Ok(())
}

pub fn run() -> Result<(), String> {
    unsafe {
        JLINK_API.JLINKARM_Go();
    }
    Ok(())
}

pub fn read_addr(addr: u64, length: usize) -> Result<Vec<u8>, String> {
    let mut data = vec![0_u8; length];
    let slice = data.as_mut_slice();
    let ptr = slice.as_mut_ptr() as *mut c_void;
    let stuff = unsafe {
        JLINK_API.JLINK_ReadMem(addr as c_uint, length as u32, ptr) as i32
    };
    println!("JLINK_ReadMem returned {}", stuff);
    Ok(data)
}

pub fn halt() -> Result<(), String> {
    let res = unsafe { JLINK_API.JLINKARM_Halt() };
    if res == 0 {
        Ok(())
    } else {
        Err(format!("Could not halt target."))
    }
}

pub fn register_list() -> Result<Vec<u32>, String> {
    const MAX_NUM_REGS: usize = 256;
    let mut regs = [0 as c_uint; MAX_NUM_REGS];
    let num_regs = unsafe {
        JLINK_API.JLINKARM_GetRegisterList(regs.as_mut_ptr(), MAX_NUM_REGS as c_int)
    };
    if num_regs < 0 || num_regs > MAX_NUM_REGS as c_int {
        return Err(format!("Got invalid number of registers from JLink."));
    }
    let num_regs = num_regs as usize;
    Ok(regs[0 .. num_regs].iter().map(|x| *x as u32).collect())
}

pub fn get_register_name_from_index(idx: u32) -> Result<String, String> {
    let cstr = unsafe {
        let result = JLINK_API.JLINKARM_GetRegisterName(idx);
        if result.is_null() {
            return Err(format!("Invalid register name for register with index {}.", idx));
        }
        CStr::from_ptr(result).to_owned()
    };
    Ok(cstr.to_str().unwrap().to_string())
}

pub fn write_register(idx: u32, value: u32) -> Result<(), String> {
    unsafe {
        let res = JLINK_API.JLINKARM_WriteReg(idx, value);
        if res != 0 {
            return Err(format!("Error writing to register {}", idx));
        }
    }
    Ok(())
}

pub fn clear_all_breakpoints() -> Result<(), String> {
    unsafe {
        let result= JLINK_API.JLINKARM_ClrBPEx(-1);
        if result != 0 {
            return Err(format!("Error clearing all breakpoints."));
        }
    }
    Ok(())
}

pub fn is_target_halted() -> Result<bool, String> {
    todo!()
}

pub fn write_to_ram(addr: u32, data: &[u8]) -> Result<(), String> {
    todo!()
}

pub fn read_from_ram(addr: u32, len: usize) -> Result<Vec<u8>, String> {
    todo!()
}

