use std::error::Error;

use super::{ Feeds, FeedsId, FeedsEntry };
use crate::{ Queries, Binds, FromRow };

impl Queries for Feeds {
    
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

impl Binds for (Option<FeedsId>, Option<&FeedsEntry>) {
    
    fn insert(self, statement: &mut sqlite::Statement) -> Result<(), Box<dyn Error>> {
        let (_, entry) = self;
        
        let entry = entry.ok_or("Entry not provided")?;
        
        statement.bind((":url", entry.url()))?;
        
        Ok(())
    }
    
    fn update(self, statement: &mut sqlite::Statement) -> Result<(), Box<dyn Error>> {
        let (id, entry) = self;
        
        let id = id.ok_or("Id not provided")?;
        let entry = entry.ok_or("Entry not provided")?;
        
        statement.bind_iter::<_, (_, sqlite::Value)>([
            (":url", entry.url().into()),
            (":id", id.to_int().into()),
        ])?;
        
        Ok(())
    }
    
    fn delete(self, statement: &mut sqlite::Statement) -> Result<(), Box<dyn Error>> {
        let (id, _) = self;
        
        let id = id.ok_or("Id not provided")?;
        
        statement.bind((":id", id.to_int()))?;
        
        Ok(())
    }
    
}

impl FromRow for (FeedsId, FeedsEntry) {
    
    fn from_row(row: sqlite::Row) -> Option<(FeedsId, FeedsEntry)> {
        let id = FeedsId::from(row.try_read::<i64, _>("id").ok()?);
        let entry = FeedsEntry::new()
            .with_url(row.try_read::<&str, _>("url").ok()?.to_string());
        
        Some((id, entry))
    }
    
}
