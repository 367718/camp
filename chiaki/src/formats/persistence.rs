use std::error::Error;

use super::{ Formats, FormatsId, FormatsEntry };
use crate::{ PersistenceQueries, PersistenceBinds, FromRow };

impl PersistenceQueries for Formats {
    
    fn create(&self) -> &str {
        "CREATE TABLE IF NOT EXISTS formats (
            id INTEGER PRIMARY KEY, 
            name TEXT
        );"
    }
    
    fn count(&self) -> &str {
        "SELECT COUNT(*) FROM formats;"
    }
    
    fn select(&self) -> &str {
        "SELECT 
        id, 
        name 
        FROM formats"
    }
    
    fn insert(&self) -> &str {
        "INSERT INTO formats 
        (name) 
        VALUES 
        (:name) 
        RETURNING id;"
    }
    
    fn update(&self) -> &str {
        "UPDATE formats SET 
        name = :name 
        WHERE id = :id;"
    }
    
    fn delete(&self) -> &str {
        "DELETE FROM formats WHERE id = :id;"
    }
    
}

impl PersistenceBinds for FormatsId {
    
    fn insert(&self, _statement: &mut sqlite::Statement) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
    
    fn update(&self, statement: &mut sqlite::Statement) -> Result<(), Box<dyn Error>> {
        statement.bind((":id", self.as_int()))?;
        
        Ok(())
    }
    
    fn delete(&self, statement: &mut sqlite::Statement) -> Result<(), Box<dyn Error>> {
        statement.bind((":id", self.as_int()))?;
        
        Ok(())
    }
    
}

impl PersistenceBinds for FormatsEntry {
    
    fn insert(&self, statement: &mut sqlite::Statement) -> Result<(), Box<dyn Error>> {
        statement.bind((":name", self.name()))?;
        
        Ok(())
    }
    
    fn update(&self, statement: &mut sqlite::Statement) -> Result<(), Box<dyn Error>> {
        statement.bind((":name", self.name()))?;
        
        Ok(())
    }
    
    fn delete(&self, _statement: &mut sqlite::Statement) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
    
}

impl FromRow for FormatsId {
    
    fn from_row(row: &sqlite::Row) -> Option<Self> {
        Some(Self::from(row.try_read::<i64, _>("id").ok()?))
    }
    
}

impl FromRow for FormatsEntry {
    
    fn from_row(row: &sqlite::Row) -> Option<Self> {
        let entry = Self::new()
            .with_name(row.try_read::<&str, _>("name").ok()?.to_string());
        
        Some(entry)
    }
    
}
