use crate::exec::Value;

use nix::unistd;
use nix::libc;


pub enum Syscall {
    Read,
    Write,
    Open,
    Close,

    Unknown,
}

impl From<u32> for Syscall {
    fn from(syscall: u32) -> Syscall {
        match syscall {
            0 => Syscall::Read,
            1 => Syscall::Write,
            2 => Syscall::Open,
            3 => Syscall::Close,
            _ => Syscall::Unknown,
        }
    }
}

fn write_temp(temp: &[u8], buf: &mut [Value]) {
    for (index, byte) in temp.iter().enumerate() {
        buf[index] = Value::Int(*byte as u32);
    }
}

fn value_to_u8(values: &[Value]) -> Vec<u8> {
    values
        .iter()
        .map(|value| value.as_int().clamp(0, 255) as u8)
        .collect::<Vec<u8>>()
}

pub fn read(fd: usize, buf: &mut [Value]) -> Result<(), Box<dyn std::error::Error>> {
    let mut temp = vec![0u8; buf.len()];
    unistd::read(fd as i32, &mut temp)?;
    write_temp(&temp, buf);

    Ok(())
}

pub fn write(fd: usize, buf: &[Value]) -> Result<(), Box<dyn std::error::Error>> {
    unistd::write(fd as i32, &value_to_u8(buf))?;

    Ok(())
}

pub fn open(filename: &str, flags: usize) -> Result<i32, i32> {
    let status = unsafe { libc::open(&(filename.as_bytes()[0] as i8) as *const i8, flags as i32) };

    return if status < 0 {
        Err(status)
    } else {
        Ok(status)
    };
}

pub fn close(fd: usize) -> Result<i32 , i32> {
    let status = unsafe { libc::close(fd as i32) };

    return if status < 0 {
        Err(status)
    } else {
        Ok(status)
    };
}



