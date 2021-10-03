use std::{fs, io};
use log::{info, error, trace, debug};
use std::fs::File;
use std::io::{Read};
use byteorder::{ReadBytesExt, LittleEndian, ByteOrder};
use std::{fmt::Write, num::ParseIntError};
use std::ptr::write_bytes;

const MAGIC: u32 = 0xD9B4BEF9;

pub trait PrivReadBytes: ReadBytesExt {

    #[inline]
    fn read_varint<T: ByteOrder>(&mut self) -> u64 {
        // 1 byte
        let mut tx_counter : u64 = self.read_u8().unwrap() as u64;

        if tx_counter == 0xFD {
            // 2 bytes
            tx_counter = self.read_u16::<T>().unwrap() as u64;

        } else if tx_counter == 0xFE {
            // 4 bytes
            tx_counter = self.read_u32::<T>().unwrap() as u64;

        } else if tx_counter == 0xFF {
            // 8 bytes
            tx_counter = self.read_u64::<T>().unwrap();
        }
        tx_counter
    }
}

impl<R: io::Read + ?Sized> PrivReadBytes for R {}

fn parseBlock(s: &str) {

    info!("Opening block {:?}", s);

    let mut f = File::open(s).expect("no file found");
    let mut size = fs::metadata(s).expect("unable to read metadata").len();

    while size != 0 {
        // 4 bytes
        let magic = f.read_u32::<LittleEndian>().unwrap();
        trace!("Magic: {:08X}", magic);

        if magic != MAGIC {
            error!("Magic value of block is incorrect!");
            break;
        }

        // 4 bytes
        let block_size = f.read_u32::<LittleEndian>().unwrap();
        trace!("Block size: {}", block_size);

        // 4 bytes
        let block_version = f.read_u32::<LittleEndian>().unwrap();
        trace!("Block version: {}", block_version);

        // 32 bytes
        let mut hash_prev_block = vec![0; 32];
        f.read(&mut hash_prev_block);

        hash_prev_block = hash_prev_block.iter().rev().cloned().collect();

        let mut hash_prev_block_str = String::with_capacity(hash_prev_block.len() * 2);
        for b in &hash_prev_block {
            write!(&mut hash_prev_block_str, "{:02x}", b).unwrap();
        }
        trace!("Hash previous block: {}", hash_prev_block_str);

        // 32 bytes
        let mut hash_merkle_root = vec![0; 32];
        f.read(&mut hash_merkle_root);

        hash_merkle_root = hash_merkle_root.iter().rev().cloned().collect();

        // 4 bytes
        let mut timestamp = f.read_u32::<LittleEndian>().unwrap();
        trace!("Timestamp: {}", timestamp);

        // 4 bytes
        let mut bits = f.read_u32::<LittleEndian>().unwrap();
        trace!("Bits (Difficulty): {:08X}", bits);

        // 4 bytes
        let mut nonce = f.read_u32::<LittleEndian>().unwrap();
        trace!("Nonce: {:08X}", nonce);

        let tx_counter = f.read_varint::<LittleEndian>();
        debug!("Tx counter: {}", tx_counter);

        // We process transactions here
        for n in 0..tx_counter {

            // 4 bytes
            let tx_version = f.read_u32::<LittleEndian>().unwrap();
            trace!("Tx {:4} version: {}", n, tx_version);

            // 2 bytes - optional
            // let flag = f.read_u16::<LittleEndian>().unwrap();
            // trace!("Tx {:4} flag: {:04X}", n, flag);
            let flag = 0u32;

            // 4 bytes - in counter if first two bytes are 00 01 then we need to look for segregated witness
            let in_counter = f.read_varint::<LittleEndian>();
            trace!("Tx {:4} in_counter: {}", n, in_counter);

            for k in 0..in_counter {

                // 32 bytes

                let mut hash_prev_tx = vec![0; 32];
                f.read(&mut hash_prev_tx);

                hash_prev_tx = hash_prev_tx.iter().rev().cloned().collect();

                let mut hash_prev_tx_str = String::with_capacity(hash_prev_tx.len() * 2);
                for b in &hash_prev_tx {
                    write!(&mut hash_prev_tx_str, "{:02x}", b).unwrap();
                }
                trace!("Hash previous tx: {}", hash_prev_block_str);

                let tx_out_index = f.read_u32::<LittleEndian>().unwrap();
                trace!("Transaction out index: {:8X}", tx_out_index);

                let tx_in_script_length = f.read_varint::<LittleEndian>();
                trace!("Tx Script Lenght: {}", tx_in_script_length);

                let mut tx_in_script = Vec::with_capacity(tx_in_script_length as usize);
                let mut tx_in_script_str = String::with_capacity(tx_in_script.len() * 2);

                tx_in_script.resize(tx_in_script_length as usize, 0);

                f.read(& mut tx_in_script);
                for b in &tx_in_script{
                    write!(&mut tx_in_script_str, "{:02x}", b).unwrap();
                }
                trace!("Script: {}", tx_in_script_str);

                // 4 bytes
                let mut sequence_no = f.read_u32::<LittleEndian>().unwrap();
                trace!("Sequence_no {:8X}", sequence_no);
            }

            let out_counter = f.read_varint::<LittleEndian>();
            trace!("Tx {:4} out_counter: {}", n, out_counter);

            for k in 0..out_counter {
                // 8 bytes
                let satoshi = f.read_u64::<LittleEndian>().unwrap();
                trace!("Value: {}", satoshi);

                let script_length = f.read_varint::<LittleEndian>();
                trace!("Script length: {}", script_length);

                let mut tx_out_script = Vec::with_capacity(script_length as usize);
                tx_out_script.resize(script_length as usize, 0);
                f.read(&mut tx_out_script);

                let mut tx_out_script_str = String::with_capacity(tx_out_script.len());
                for b in &tx_out_script{
                    write!(&mut tx_out_script_str, "{:02x}", b).unwrap();
                }
                trace!("Out Script: {}", tx_out_script_str);


            }


            if flag != 0 {
                // segregated witness

            }

            let mut lock_time = f.read_u32::<LittleEndian>().unwrap();
            trace!("Lock time : {}", lock_time);

        }

        size -= (block_size+8) as u64;
    }


}

fn main() -> io::Result<()> {

    env_logger::init();

    info!(target:"info", "Starting block parsing ... ");

    let mut entries = fs::read_dir("/run/media/igor/b72da028-f6f8-4ac4-80ae-5a59a97de881/btc/blocks")?
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, io::Error>>()?;

    // The order in which `read_dir` returns entries is not guaranteed. If reproducible
    // ordering is required the entries should be explicitly sorted.

    entries.sort();

    // The entries have now been sorted by their path.

    for i in entries {
        parseBlock(i.as_os_str().to_str().unwrap());
    }

    Ok(())
}