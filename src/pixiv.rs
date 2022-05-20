use lazy_static::lazy_static;
use regex::Regex;
use sanitize_filename::sanitize;
use std::fs::{File, OpenOptions};
use std::io::ErrorKind;
use std::{io::Write, path::Path};

// Single Novel
pub struct Novel {
    content: String,
    author: String,
    title: String,
    pid: String,
    uid: String,
}

impl Novel {
    pub fn new(title: String, author: String, content: String, pid: String, uid: String) -> Novel {
        Novel {
            content: Novel::post_process(&content),
            author,
            title,
            pid,
            uid,
        }
    }

    fn get_filename(&self) -> String {
        format!("{}-{}.txt", self.title, self.author)
    }

    pub fn save(&self) {
        let filename = sanitize(self.get_filename());
        let path = Path::new(&filename);
        println!("[Download]: {}", &self.title);
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)
            .expect("Failed to create file.");
        write!(
            &file,
            "pid:{}\ntitle:{}\nauthor:{}[{}]\n==================\n",
            self.pid, self.title, self.author, self.uid
        )
        .expect("Failed to write file");

        file.write_all(self.content.as_ref())
            .expect("Failed to write.");
    }

    pub fn post_process(content: &str) -> String {
        lazy_static! {
            static ref REPLACE_RE: Regex =
                Regex::new(r"\[newpage\]|\[chapter.+\]|\[uploadedimage.+\]").unwrap();
        }
        REPLACE_RE.replace_all(content, "").to_string()
    }
}

// Series
pub struct Series {
    // title: String,
    // author: String,
    file: File,
    pub pid: String,
    // uid: String,
}

impl Series {
    pub fn new(title: String, author: String, pid: String, uid: String) -> Series {
        // check whether update is available
        let file = OpenOptions::new()
            .write(true)
            .append(true)
            .create(true)
            .open(sanitize(format!("{}-{}.txt", title, author)))
            .expect("Failed to create file.");
        write!(
            &file,
            "pid:{}\nseries:{}\nauthor:{}[{}]\n==================\n",
            pid, title, author, uid
        )
        .expect("Failed to write file.");
        Series {
            // title,
            // author,
            file,
            pid,
            // uid,
        }
    }

    pub fn append(&mut self, novel: Novel) {
        println!("[Download]: {}", &novel.title);
        self.file
            .write_all(format!("###{}\n{}\n", novel.title, novel.content).as_ref())
            .expect("Failed to append content.");
    }
}
