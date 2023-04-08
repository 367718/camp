use std::error::Error;

use super::{ Kinds, KindsId, KindsEntry };
use crate::{ Queries, Binds, FromRow };

impl Queries for Kinds {
    
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

impl Binds for (Option<KindsId>, Option<&KindsEntry>) {
    
    fn insert(self, statement: &mut sqlite::Statement) -> Result<(), Box<dyn Error>> {
        let (_, entry) = self;
        
        let entry = entry.ok_or("Entry not provided")?;
        
        statement.bind((":name", entry.name()))?;
        
        Ok(())
    }
    
    fn update(self, statement: &mut sqlite::Statement) -> Result<(), Box<dyn Error>> {
        let (id, entry) = self;
        
        let id = id.ok_or("Id not provided")?;
        let entry = entry.ok_or("Entry not provided")?;
        
        statement.bind_iter::<_, (_, sqlite::Value)>([
            (":name", entry.name().into()),
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

impl FromRow for (KindsId, KindsEntry) {
    
    fn from_row(row: sqlite::Row) -> Option<(KindsId, KindsEntry)> {
        let id = KindsId::from(row.try_read::<i64, _>("id").ok()?);
        let entry = KindsEntry::new()
            .with_name(row.try_read::<&str, _>("name").ok()?.to_string());
        
        Some((id, entry))
    }
    
}
