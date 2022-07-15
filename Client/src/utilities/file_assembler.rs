use std::{
    fs::{File, OpenOptions},
    io::{Read, Write},
};

use crate::errors::download_manager_error::DownloadManagerError;
const BUFFERED_PIECE_QUANTITY: usize = 20;

/// Creates a file at given path, and appends quantity files which paths are in the next format ["path/piece_{index}.txt"]
pub fn assemble(
    src_dir: String,
    dst_path: String,
    quantity: usize,
    piece_lenght: usize,
) -> Result<(), DownloadManagerError> {
    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(dst_path)?;

    let buffer_len = if quantity >= BUFFERED_PIECE_QUANTITY {
        BUFFERED_PIECE_QUANTITY
    } else {
        quantity
    };
    let mut buffer: Vec<u8> = Vec::with_capacity(buffer_len * piece_lenght);

    for i in 0..quantity {
        let piece_path = format!("{}/piece_{}.txt", src_dir, i);
        let mut piece_file = File::open(piece_path)?;
        let mut piece_content = Vec::new();
        piece_file.read_to_end(&mut piece_content)?;

        buffer.extend(&piece_content);

        if i % buffer_len == 0 && i != 0 {
            file.write_all(&buffer)?;
            file.flush()?;
            buffer.clear();
        }
    }
    if !buffer.is_empty() {
        file.write_all(&buffer)?;
        file.flush()?;
        buffer.clear();
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs::remove_file, path::Path};

    #[test]
    fn test_assemble_40_pieces() {
        let src_dir =
            "src/test_files/piece_assembler_test_files/debian-edu-11.3.0-amd64-netinst.iso.torrent";
        let dst_path = "src/test_files/piece_assembler_test_files/assembled_test_1.txt";
        let _r = remove_file(Path::new(dst_path));

        let quantity = 40;
        let piece_lenght = 262144;
        assert!(assemble(
            src_dir.to_string(),
            dst_path.to_string(),
            quantity,
            piece_lenght
        )
        .is_ok());
        let mut file = File::open(dst_path).unwrap();
        let mut content = Vec::new();
        file.read_to_end(&mut content).unwrap();
        assert_eq!(content.len(), quantity * piece_lenght);
    }
    #[test]
    fn test_assemble_5_pieces() {
        let src_dir =
            "src/test_files/piece_assembler_test_files/debian-edu-11.3.0-amd64-netinst.iso.torrent";
        let dst_path = "src/test_files/piece_assembler_test_files/assembled_test_2.txt";
        let _r = remove_file(Path::new(dst_path));

        let quantity = 5;
        let piece_lenght = 262144;
        assert!(assemble(
            src_dir.to_string(),
            dst_path.to_string(),
            quantity,
            piece_lenght
        )
        .is_ok());
        let mut file = File::open(dst_path).unwrap();
        let mut content = Vec::new();
        file.read_to_end(&mut content).unwrap();
        assert_eq!(content.len(), quantity * piece_lenght);
    }
}
