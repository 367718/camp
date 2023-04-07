use std::{
    error::Error,
    fs::{ self, OpenOptions },
    path::Path,
};

pub struct Persistence {
    connection: sqlite::Connection,
}

impl Persistence {
    
    // ---------- constructors ----------
    
    
    pub fn new<S: AsRef<Path>>(path: S) -> Result<Self, Box<dyn Error>> {
        let path = path.as_ref();
        
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)
            .map_err(|_| format!("File already exists or write error: {}", path.to_string_lossy()))?;
        
        let flags = sqlite::OpenFlags::new()
            .set_read_write();
        
        Ok(Self {
            connection: sqlite::Connection::open_with_flags(path, flags)?,
        })
    }
    
    pub fn load<S: AsRef<Path>>(path: S) -> Result<Self, Box<dyn Error>> {
        let path = path.as_ref();
        
        let flags = sqlite::OpenFlags::new()
            .set_read_write();
        
        Ok(Self {
            connection: sqlite::Connection::open_with_flags(path, flags)?,
        })
    }
    
    
    // ---------- accessors ----------
    
    
    pub fn count(&self, queries: &impl PersistenceQueries) -> Result<i64, Box<dyn Error>> {
        let mut statement = self.connection.prepare(queries.count())?;
        
        statement.next()?;
        
        Ok(statement.read::<i64, _>(0)?)
    }
    
    pub fn select<'p, I: FromRow, E: FromRow>(&'p self, queries: &impl PersistenceQueries) -> Result<impl Iterator<Item = (I, E)> + 'p, Box<dyn Error>> {
        
        fn from_row<I: FromRow, E: FromRow>(row: &sqlite::Row) -> Option<(I, E)> {
            Some((I::from_row(row)?, E::from_row(row)?))
        }
        
        let entries = self.connection
            .prepare(queries.select())?
            .into_iter()
            .filter_map(Result::ok)
            .filter_map(|row| from_row(&row));
        
        Ok(entries)
        
    }
    
    
    // ---------- mutators ----------
    
    
    pub fn create(&mut self, queries: &impl PersistenceQueries) -> Result<(), Box<dyn Error>> {
        Ok(self.connection.execute(queries.create())?)
    }
    
    pub fn insert(&mut self, queries: &impl PersistenceQueries, entry: &impl PersistenceBinds) -> Result<i64, Box<dyn Error>> {
        let mut statement = self.connection.prepare(queries.insert())?;
        
        entry.insert(&mut statement)?;
        
        statement.next()?;
        
        Ok(statement.read::<i64, _>(0)?)
    }
    
    pub fn update(&mut self, queries: &impl PersistenceQueries, entry: &impl PersistenceBinds, id: impl PersistenceBinds) -> Result<(), Box<dyn Error>> {
        let mut statement = self.connection.prepare(queries.update())?;
        
        entry.update(&mut statement)?;
        id.update(&mut statement)?;
        
        statement.next()?;
        
        if self.connection.change_count() == 0 {
            return Err("Database update operation failed".into());
        }
        
        Ok(())
    }
    
    pub fn delete(&mut self, queries: &impl PersistenceQueries, id: impl PersistenceBinds) -> Result<(), Box<dyn Error>> {
        let mut statement = self.connection.prepare(queries.delete())?;
        
        id.delete(&mut statement)?;
        
        statement.next()?;
        
        if self.connection.change_count() == 0 {
            return Err("Database delete operation failed".into());
        }
        
        Ok(())
    }
    
    pub fn begin_transaction(&mut self) -> Result<(), Box<dyn Error>> {
        Ok(self.connection.execute("BEGIN;")?)
    }
    
    pub fn commit(&mut self) {
        self.connection.execute("COMMIT;").expect("Database transaction could not be commited.");
    }
    
    pub fn rollback(&mut self) {
        self.connection.execute("ROLLBACK;").expect("Database transaction could not be rolled back.");
    }
    
}

pub trait PersistenceQueries {
    
    fn create(&self) -> &str;
    
    fn count(&self) -> &str;
    
    fn select(&self) -> &str;
    
    fn insert(&self) -> &str;
    
    fn update(&self) -> &str;
    
    fn delete(&self) -> &str;
    
}

pub trait PersistenceBinds {
    
    fn insert(&self, statement: &mut sqlite::Statement) -> Result<(), Box<dyn Error>>;
    
    fn update(&self, statement: &mut sqlite::Statement) -> Result<(), Box<dyn Error>>;
    
    fn delete(&self, statement: &mut sqlite::Statement) -> Result<(), Box<dyn Error>>;
    
}

pub trait FromRow {
    
    fn from_row(row: &sqlite::Row) -> Option<Self> where Self: Sized;
    
}
