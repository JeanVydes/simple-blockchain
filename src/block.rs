use bson::SerializerOptions;
use crate::models::{self, Block};
use std::{fs, io::Write, path::Path, time::SystemTime};

pub fn register_block_locally(workdir: String, block: models::Block) -> std::io::Result<()> {
    let file_path = format!("{}/{}.block", workdir, block.index);
    let file = std::fs::File::create(file_path)?;

    let serializer_options = SerializerOptions::builder().human_readable(false).build();
    let bson_doc = bson::to_document_with_options(&block, serializer_options);

    let mut writer = std::io::BufWriter::new(file);
    bson_doc.unwrap().to_writer(&mut writer).unwrap();
    writer.flush()?;

    Ok(())
}

pub fn get_block_by_index(index: u64, workdir: &Path) -> std::io::Result<Block> {
    let raw_file_path = format!("{}/{}.block", workdir.to_string_lossy(), index);
    let raw_file = std::fs::File::open(raw_file_path)?;
    let mut reader = std::io::BufReader::new(raw_file);
    let bson_doc = bson::Document::from_reader(&mut reader);

    // shit code
    match bson_doc {
        Ok(doc) => {
            let raw_block = bson::from_document(doc);
            match raw_block {
                Ok(block) => {
                    return Ok(block);
                },
                Err(e) => {
                    return Err(std::io::Error::new(std::io::ErrorKind::Other, e));
                }
            }
        },
        Err(e) => {
            return Err(std::io::Error::new(std::io::ErrorKind::Other, e));
        }
    }
}

pub fn get_newest_block(folder: &Path) -> std::io::Result<Block> {
    if folder.is_dir() {
        let mut newest_file = None;
        let mut newest_time = SystemTime::UNIX_EPOCH;

        for entry in fs::read_dir(folder).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_file() {
                let metadata = fs::metadata(&path).unwrap();
                let modified_time = metadata.modified().unwrap();
                if modified_time > newest_time {
                    newest_time = modified_time;
                    newest_file = Some(path);
                }
            }
        }

        match newest_file {
            Some(path) => {
                let genesis_block_file = std::fs::File::open(path).unwrap();
                let mut reader = std::io::BufReader::new(genesis_block_file);
                let bson_doc = bson::Document::from_reader(&mut reader).unwrap();
                let result = bson::from_document(bson_doc);
                match result {
                    Ok(block) => Ok(block),
                    Err(e) => {
                        println!("Error: {}", e);
                        Err(std::io::Error::new(std::io::ErrorKind::Other, e))
                    }
                }
            },
            None => return Err(std::io::Error::new(std::io::ErrorKind::Other, "No blocks found")),
        }
    }else {
        return Err(std::io::Error::new(std::io::ErrorKind::Other, "No blocks found"));
    }
}