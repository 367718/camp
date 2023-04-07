use std::error::Error;

use super::{ Kinds, KindsId, KindsEntry };
use crate::{ PersistenceQueries, PersistenceBinds, FromRow };

impl PersistenceQueries for Kinds {
    
    fn create(&self) -> &str {
        "CREATE TABLE IF NOT EXISTS kinds (
            id INTEGER PRIMARY KEY, 
            name TEXT
        );"
    }
    
    fn count(&self) -> &str {
        "SELECT COUNT(*) FROM kinds;"
    }
    
    fn select(&self) -> &str {
        "SELECT 
        id, 
        name 
        FROM kinds"
    }
    
    fn insert(&self) -> &str {
        "INSERT INTO kinds 
        (name) 
        VALUES 
        (:name) 
        RETURNING id;"
    }
    
    fn update(&self) -> &str {
        "UPDATE kinds SET 
        name = :name 
        WHERE id = :id;"
    }
    
    fn delete(&self) -> &str {
        "DELETE FROM kinds WHERE id = :id;"
    }
    
}

impl PersistenceBinds for KindsId {
    
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

impl PersistenceBinds for KindsEntry {
    
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

impl FromRow for KindsId {
    
    fn from_row(row: &sqlite::Row) -> Option<Self> {
        Some(Self::from(row.try_read::<i64, _>("id").ok()?))
    }
    
}

impl FromRow for KindsEntry {
    
    fn from_row(row: &sqlite::Row) -> Option<Self> {
        let entry = Self::new()
            .with_name(row.try_read::<&str, _>("name").ok()?.to_string());
        
        Some(entry)
    }
    
}
