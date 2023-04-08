use std::error::Error;

use super::{ Feeds, FeedsId, FeedsEntry };
use crate::{ PersistenceQueries, PersistenceBinds, FromRow };

impl PersistenceQueries for Feeds {
    
    fn create(&self) -> &str {
        "CREATE TABLE IF NOT EXISTS feeds (
            id INTEGER PRIMARY KEY, 
            url TEXT
        );"
    }
    
    fn count(&self) -> &str {
        "SELECT COUNT(*) FROM feeds;"
    }
    
    fn select(&self) -> &str {
        "SELECT 
        id, 
        url 
        FROM feeds"
    }
    
    fn insert(&self) -> &str {
        "INSERT INTO feeds 
        (url) 
        VALUES 
        (:url) 
        RETURNING id;"
    }
    
    fn update(&self) -> &str {
        "UPDATE feeds SET 
        url = :url 
        WHERE id = :id;"
    }
    
    fn delete(&self) -> &str {
        "DELETE FROM feeds WHERE id = :id;"
    }
    
}

impl PersistenceBinds for FeedsId {
    
    fn insert(&self, _statement: &mut sqlite::Statement) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
    
    fn update(&self, statement: &mut sqlite::Statement) -> Result<(), Box<dyn Error>> {
        statement.bind((":id", i64::from(*self)))?;
        
        Ok(())
    }
    
    fn delete(&self, statement: &mut sqlite::Statement) -> Result<(), Box<dyn Error>> {
        statement.bind((":id", i64::from(*self)))?;
        
        Ok(())
    }
    
}

impl PersistenceBinds for FeedsEntry {
    
    fn insert(&self, statement: &mut sqlite::Statement) -> Result<(), Box<dyn Error>> {
        statement.bind((":url", self.url()))?;
        
        Ok(())
    }
    
    fn update(&self, statement: &mut sqlite::Statement) -> Result<(), Box<dyn Error>> {
        statement.bind((":url", self.url()))?;
        
        Ok(())
    }
    
    fn delete(&self, _statement: &mut sqlite::Statement) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
    
}

impl FromRow for FeedsId {
    
    fn from_row(row: &sqlite::Row) -> Option<Self> {
        Some(Self::from(row.try_read::<i64, _>("id").ok()?))
    }
    
}

impl FromRow for FeedsEntry {
    
    fn from_row(row: &sqlite::Row) -> Option<Self> {
        let entry = Self::new()
            .with_url(row.try_read::<&str, _>("url").ok()?.to_string());
        
        Some(entry)
    }
    
}
