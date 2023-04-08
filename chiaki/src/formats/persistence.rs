use std::error::Error;

use super::{ Formats, FormatsId, FormatsEntry };
use crate::{ Queries, Binds, FromRow };

impl Queries for Formats {
    
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

impl Binds for (Option<FormatsId>, Option<&FormatsEntry>) {
    
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

impl FromRow for (FormatsId, FormatsEntry) {
    
    fn from_row(row: sqlite::Row) -> Option<(FormatsId, FormatsEntry)> {
        let id = FormatsId::from(row.try_read::<i64, _>("id").ok()?);
        let entry = FormatsEntry::new()
            .with_name(row.try_read::<&str, _>("name").ok()?.to_string());
        
        Some((id, entry))
    }
    
}
