mod feeds;
mod formats;
mod kinds;
mod series;
mod candidates;

use std::{
    error::Error,
    ffi::OsString,
    fs::{ self, File, OpenOptions },
    path::Path,
};

use feeds::Feeds;
use formats::Formats;
use kinds::Kinds;
use series::Series;
use candidates::Candidates;

pub use feeds::{ FeedsId, FeedsEntry };
pub use formats::{ FormatsId, FormatsEntry };
pub use kinds::{ KindsId, KindsEntry };
pub use series::{ SeriesId, SeriesEntry, SeriesStatus, SeriesGood };
pub use candidates::{ CandidatesId, CandidatesEntry, CandidatesCurrent };

use bincode::{ Decode, Encode };

pub struct Database {
    data: Data,
    modified: bool,
}

#[derive(Decode, Encode)]
struct Data {
    feeds: Feeds,
    formats: Formats,
    kinds: Kinds,
    series: Series,
    candidates: Candidates,
}

const DATABASE_SIZE_LIMIT: usize = 50 * 1024 * 1024;

impl Database {
    
    // ---------- constructors ----------
    
    
    pub fn new<S: AsRef<Path>>(path: S) -> Result<Self, Box<dyn Error>> {
        let mut database = Self {
            data: Data {
                feeds: Feeds::new(),
                formats: Formats::new(),
                kinds: Kinds::new(),
                series: Series::new(),
                candidates: Candidates::new(),
            },
            modified: true,
        };
        
        database.save(path)?;
        
        Ok(database)
    }
    
    pub fn load<S: AsRef<Path>>(path: S) -> Result<Self, Box<dyn Error>> {
        let config = bincode::config::standard()
            .with_little_endian()
            .with_fixed_int_encoding()
            .with_limit::<DATABASE_SIZE_LIMIT>();
        
        let database = Self {
            data: bincode::decode_from_std_read(&mut File::open(path)?, config)?,
            modified: false,
        };
        
        Ok(database)
    }
    
    
    // ---------- mutators ----------
    
    
    pub fn save<S: AsRef<Path>>(&mut self, path: S) -> Result<(), Box<dyn Error>> {
        if self.modified {
            
            let path = path.as_ref();
            
            let extension = if let Some(current) = path.extension() {
                let mut composite = OsString::with_capacity(current.len() + 4);
                composite.push(current);
                composite.push(".tmp");
                composite
            } else {
                OsString::from("tmp")
            };
            
            let tmp_path = path.with_extension(extension);
            
            let mut tmp_file = OpenOptions::new()
                .write(true)
                .create(true)
                .open(&tmp_path)
                .map_err(|_| format!("Could not create file: {}", tmp_path.to_string_lossy()))?;
            
            let config = bincode::config::standard()
                .with_little_endian()
                .with_fixed_int_encoding()
                .write_fixed_array_length()
                .with_limit::<DATABASE_SIZE_LIMIT>();
            
            bincode::encode_into_std_write(&self.data, &mut tmp_file, config)?;
            
            // attempt to perform the update atomically
            fs::rename(&tmp_path, path)?;
            
            self.modified = false;
            
        }
        
        Ok(())
    }
    
}

// feeds

impl Database {
    
    // ---------- accessors ----------
    
    
    pub fn feeds_get(&self, id: FeedsId) -> Option<&FeedsEntry> {
        self.data.feeds.get(id)
    }
    
    pub fn feeds_iter(&self) -> impl Iterator<Item = (&FeedsId, &FeedsEntry)> {
        self.data.feeds.iter()
    }
    
    pub fn feeds_count(&self) -> usize {
        self.data.feeds.count()
    }
    
    
    // ---------- mutators ----------
    
    
    pub fn feeds_add(&mut self, entry: FeedsEntry) -> Result<FeedsId, Box<dyn Error>> {
        let result = self.data.feeds.add(entry)?;
        self.modified = true;
        Ok(result)
    }
    
    pub fn feeds_edit(&mut self, id: FeedsId, entry: FeedsEntry) -> Result<FeedsEntry, Box<dyn Error>> {
        let result = self.data.feeds.edit(id, entry)?;
        self.modified = true;
        Ok(result)
    }
    
    pub fn feeds_remove(&mut self, id: FeedsId) -> Result<FeedsEntry, Box<dyn Error>> {
        let result = self.data.feeds.remove(id)?;
        self.modified = true;
        Ok(result)
    }
    
}

// formats

impl Database {
    
    // ---------- accessors ----------
    
    
    pub fn formats_get(&self, id: FormatsId) -> Option<&FormatsEntry> {
        self.data.formats.get(id)
    }
    
    pub fn formats_iter(&self) -> impl Iterator<Item = (&FormatsId, &FormatsEntry)> {
        self.data.formats.iter()
    }
    
    pub fn formats_count(&self) -> usize {
        self.data.formats.count()
    }
    
    
    // ---------- mutators ----------
    
    
    pub fn formats_add(&mut self, entry: FormatsEntry) -> Result<FormatsId, Box<dyn Error>> {
        let result = self.data.formats.add(entry)?;
        self.modified = true;
        Ok(result)
    }
    
    pub fn formats_edit(&mut self, id: FormatsId, entry: FormatsEntry) -> Result<FormatsEntry, Box<dyn Error>> {
        let result = self.data.formats.edit(id, entry)?;
        self.modified = true;
        Ok(result)
    }
    
    pub fn formats_remove(&mut self, id: FormatsId) -> Result<FormatsEntry, Box<dyn Error>> {
        let result = self.data.formats.remove(id)?;
        self.modified = true;
        Ok(result)
    }
    
}

// kinds

impl Database {
    
    // ---------- accessors ----------
    
    
    pub fn kinds_get(&self, id: KindsId) -> Option<&KindsEntry> {
        self.data.kinds.get(id)
    }
    
    pub fn kinds_iter(&self) -> impl Iterator<Item = (&KindsId, &KindsEntry)> {
        self.data.kinds.iter()
    }
    
    pub fn kinds_count(&self) -> usize {
        self.data.kinds.count()
    }
    
    
    // ---------- mutators ----------
    
    
    pub fn kinds_add(&mut self, entry: KindsEntry) -> Result<KindsId, Box<dyn Error>> {
        let result = self.data.kinds.add(entry)?;
        self.modified = true;
        Ok(result)
    }
    
    pub fn kinds_edit(&mut self, id: KindsId, entry: KindsEntry) -> Result<KindsEntry, Box<dyn Error>> {
        let result = self.data.kinds.edit(id, entry)?;
        self.modified = true;
        Ok(result)
    }
    
    pub fn kinds_remove(&mut self, id: KindsId) -> Result<KindsEntry, Box<dyn Error>> {
        let result = self.data.kinds.remove(id, &self.data.series)?;
        self.modified = true;
        Ok(result)
    }
    
}

// series

impl Database {
    
    // ---------- accessors ----------
    
    
    pub fn series_get(&self, id: SeriesId) -> Option<&SeriesEntry> {
        self.data.series.get(id)
    }
    
    pub fn series_iter(&self) -> impl Iterator<Item = (&SeriesId, &SeriesEntry)> {
        self.data.series.iter()
    }
    
    pub fn series_count(&self) -> usize {
        self.data.series.count()
    }
    
    
    // ---------- mutators ----------
    
    
    pub fn series_add(&mut self, entry: SeriesEntry) -> Result<SeriesId, Box<dyn Error>> {
        let result = self.data.series.add(entry, &self.data.kinds, &self.data.candidates)?;
        self.modified = true;
        Ok(result)
    }
    
    pub fn series_edit(&mut self, id: SeriesId, entry: SeriesEntry) -> Result<SeriesEntry, Box<dyn Error>> {
        let result = self.data.series.edit(id, entry, &self.data.kinds, &self.data.candidates)?;
        self.modified = true;
        Ok(result)
    }
    
    pub fn series_remove(&mut self, id: SeriesId) -> Result<SeriesEntry, Box<dyn Error>> {
        let result = self.data.series.remove(id, &self.data.candidates)?;
        self.modified = true;
        Ok(result)
    }
    
}

// candidates

impl Database {
    
    // ---------- accessors ----------
    
    
    pub fn candidates_get(&self, id: CandidatesId) -> Option<&CandidatesEntry> {
        self.data.candidates.get(id)
    }
    
    pub fn candidates_iter(&self) -> impl Iterator<Item = (&CandidatesId, &CandidatesEntry)> {
        self.data.candidates.iter()
    }
    
    pub fn candidates_count(&self) -> usize {
        self.data.candidates.count()
    }
    
    
    // ---------- mutators ----------
    
    
    pub fn candidates_add(&mut self, entry: CandidatesEntry) -> Result<CandidatesId, Box<dyn Error>> {
        let result = self.data.candidates.add(entry, &self.data.series)?;
        self.modified = true;
        Ok(result)
    }
    
    pub fn candidates_edit(&mut self, id: CandidatesId, entry: CandidatesEntry) -> Result<CandidatesEntry, Box<dyn Error>> {
        let result = self.data.candidates.edit(id, entry, &self.data.series)?;
        self.modified = true;
        Ok(result)
    }
    
    pub fn candidates_remove(&mut self, id: CandidatesId) -> Result<CandidatesEntry, Box<dyn Error>> {
        let result = self.data.candidates.remove(id)?;
        self.modified = true;
        Ok(result)
    }
    
}
