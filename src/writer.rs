// src/writer.rs
use std::io::{self, BufWriter, Write};
use std::fs::{File, OpenOptions};

pub trait RdfWriter {
    fn add_triple_raw(&mut self, subj: &str, pred: &str, obj: &str, obj_is_literal: bool) -> io::Result<()>;
}

pub struct FileWriter {
    writer: BufWriter<File>,
}

impl FileWriter {
    pub fn to_file(output_file: String) -> io::Result<Self> {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(output_file)?;
        Ok(FileWriter {
            writer: BufWriter::new(file),
        })
    }
}

impl RdfWriter for FileWriter {
    fn add_triple_raw(&mut self, subj: &str, pred: &str, obj: &str, obj_is_literal: bool) -> io::Result<()> {
        let obj_repr = if obj_is_literal {
            format!("\"{}\"", obj.replace('"', "\\\""))
        } else {
            obj.to_string()
        };
        self.writer.write_all(subj.as_bytes())?;
        self.writer.write_all(b" ")?;
        self.writer.write_all(pred.as_bytes())?;
        self.writer.write_all(b" ")?;
        self.writer.write_all(obj_repr.as_bytes())?;
        self.writer.write_all(b" .\n")?;
        let _ = self.writer.flush();
        Ok(())
    }
}

pub struct MemoryWriter {
    pub triples: Vec<String>,
}

impl MemoryWriter {
    pub fn new() -> Self {
        MemoryWriter { triples: Vec::new() }
    }

    pub fn into_string(self) -> String {
        self.triples.join("\n")
    }
}

impl RdfWriter for MemoryWriter {
    fn add_triple_raw(&mut self, subj: &str, pred: &str, obj: &str, obj_is_literal: bool) -> io::Result<()> {
        let obj_repr = if obj_is_literal {
            format!("\"{}\"", obj.replace('"', "\\\""))
        } else {
            obj.to_string()
        };
        self.triples.push(format!("{} {} {} .", subj, pred, obj_repr));
        Ok(())
    }
}
