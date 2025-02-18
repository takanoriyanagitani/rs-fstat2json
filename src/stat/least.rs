use std::io;

use std::io::BufWriter;
use std::io::Write;

use std::io::BufRead;

use std::path::Path;

use std::time::SystemTime;

use std::fs::Metadata;

#[derive(serde::Serialize)]
pub enum FileType {
    Unspecified,

    /// Regular file.
    Regular,

    /// Symbolic link.
    Symlink,

    /// Character device.
    Character,

    /// Block device.
    Block,

    /// Directory.
    Directory,

    /// FIFO.
    Fifo,

    /// Socket.
    Socket,

    Unknown,
}

#[derive(serde::Serialize)]
pub enum Permissions {
    ReadOnly(bool),
    Mode(u32),
}

#[derive(serde::Serialize)]
pub struct LeastStat {
    pub name: String,
    pub size: u64,
    pub mode: Permissions,
    pub modified: SystemTime,
    pub file_type: FileType,
}

#[derive(serde::Serialize)]
pub struct LeastStatJson {
    pub name: String,
    pub size: u64,
    pub mode: Permissions,
    pub modified: String,
    pub file_type: FileType,
}

pub fn converter_new() -> impl FnMut(LeastStat) -> Result<LeastStatJson, io::Error> {
    let mut buf: Vec<u8> = vec![];
    let tfmt = time::format_description::well_known::Rfc3339;

    move |l: LeastStat| {
        buf.clear();
        let o: time::OffsetDateTime = l.modified.into();
        let modified: String = o.format(&tfmt).map_err(io::Error::other)?;
        Ok(LeastStatJson {
            name: l.name,
            size: l.size,
            mode: l.mode,
            modified,
            file_type: l.file_type,
        })
    }
}

impl From<LeastStat> for LeastStatJson {
    fn from(l: LeastStat) -> Self {
        Self {
            name: l.name,
            size: l.size,
            mode: l.mode,
            modified: "".into(),
            file_type: l.file_type,
        }
    }
}

#[cfg(target_family = "unix")]
use std::os::unix::fs::PermissionsExt;

#[cfg(target_family = "unix")]
impl From<std::fs::Permissions> for Permissions {
    fn from(p: std::fs::Permissions) -> Self {
        Self::Mode(p.mode())
    }
}

#[cfg(not(target_family = "unix"))]
impl From<std::fs::Permissions> for Permissions {
    fn from(p: std::fs::Permissions) -> Self {
        Self::ReadOnly(p.readonly())
    }
}

#[cfg(target_family = "unix")]
use std::os::unix::fs::FileTypeExt;

#[cfg(target_family = "unix")]
impl From<std::fs::FileType> for FileType {
    fn from(f: std::fs::FileType) -> Self {
        if f.is_dir() {
            return Self::Directory;
        }

        if f.is_file() {
            return Self::Regular;
        }

        if f.is_symlink() {
            return Self::Symlink;
        }

        if f.is_block_device() {
            return Self::Block;
        }

        if f.is_char_device() {
            return Self::Character;
        }

        if f.is_fifo() {
            return Self::Fifo;
        }

        if f.is_socket() {
            return Self::Socket;
        }

        Self::Unknown
    }
}

#[cfg(not(target_family = "unix"))]
impl From<std::fs::FileType> for FileType {
    fn from(f: std::fs::FileType) -> Self {
        if f.is_dir() {
            return Self::Directory;
        }

        if f.is_file() {
            return Self::Regular;
        }

        if f.is_symlink() {
            return Self::Symlink;
        }

        Self::Unknown
    }
}

impl LeastStat {
    pub fn from_path<P>(p: P) -> Result<Self, io::Error>
    where
        P: AsRef<Path>,
    {
        let s: &str = p
            .as_ref()
            .to_str()
            .ok_or_else(|| io::Error::other("invalid path"))?;

        let m: Metadata = std::fs::metadata(&p)?;
        let size: u64 = m.len();
        let mode: Permissions = m.permissions().into();
        let modified: SystemTime = m.modified()?;
        let file_type: FileType = m.file_type().into();

        Ok(Self {
            name: s.into(),
            size,
            mode,
            modified,
            file_type,
        })
    }
}

pub fn stat2writer_new<W>(mut wtr: W) -> impl FnMut(LeastStat) -> Result<(), io::Error>
where
    W: Write,
{
    let mut conv = converter_new();

    move |least: LeastStat| {
        let lj: LeastStatJson = conv(least)?;
        serde_json::to_writer(&mut wtr, &lj).map_err(io::Error::other)?;
        writeln!(&mut wtr)
    }
}

pub fn filenames2stats2writer<I, W>(filenames: I, mut writer: W) -> Result<(), io::Error>
where
    I: Iterator<Item = Result<String, io::Error>>,
    W: FnMut(LeastStat) -> Result<(), io::Error>,
{
    for rname in filenames {
        let name: String = rname?;
        let least: LeastStat = LeastStat::from_path(name)?;
        writer(least)?;
    }

    Ok(())
}

pub fn stdin2filenames2stats2stdout() -> Result<(), io::Error> {
    let i = io::stdin();
    let il = i.lock();
    let lines = il.lines();

    let o = io::stdout();
    let mut ol = o.lock();

    let mut bw = BufWriter::new(&mut ol);
    filenames2stats2writer(lines, stat2writer_new(&mut bw))?;
    bw.flush()?;
    drop(bw);

    ol.flush()
}
