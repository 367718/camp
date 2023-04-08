use std::error::Error;

use super::{ Series, SeriesId, SeriesEntry, SeriesStatus, SeriesGood, KindsId };
use crate::{ Queries, Binds, FromRow };

impl Queries for Series {
    
    fn create(&self) -> &str {
        "CREATE TABLE IF NOT EXISTS series (
            id INTEGER PRIMARY KEY, 
            title TEXT, 
            kind INTEGER, 
            status INTEGER, 
            progress INTEGER, 
            good INTEGER
        );"
    }
    
    fn count(&self) -> &str {
        "SELECT COUNT(*) FROM series;"
    }
    
    fn select(&self) -> &str {
        "SELECT 
        id, 
        title, 
        kind, 
        status, 
        progress, 
        good 
        FROM series"
    }
    
    fn insert(&self) -> &str {
        "INSERT INTO series 
        (title, kind, status, progress, good) 
        VALUES 
        (:title, :kind, :status, :progress, :good) 
        RETURNING id;"
    }
    
    fn update(&self) -> &str {
        "UPDATE series SET 
        title = :title, 
        kind = :kind, 
        status = :status, 
        progress = :progress, 
        good = :good 
        WHERE id = :id;"
    }
    
    fn delete(&self) -> &str {
        "DELETE FROM series WHERE id = :id;"
    }
    
}

impl Binds for (Option<SeriesId>, Option<&SeriesEntry>) {
    
    fn insert(self, statement: &mut sqlite::Statement) -> Result<(), Box<dyn Error>> {
        let (_, entry) = self;
        
        let entry = entry.ok_or("Entry not provided")?;
        
        statement.bind_iter::<_, (_, sqlite::Value)>([
            (":title", entry.title().into()),
            (":kind", entry.kind().to_int().into()),
            (":status", entry.status().to_int().into()),
            (":progress", entry.progress().into()),
            (":good", entry.good().to_int().into()),
        ])?;
        
        Ok(())
    }
    
    fn update(self, statement: &mut sqlite::Statement) -> Result<(), Box<dyn Error>> {
        let (id, entry) = self;
        
        let id = id.ok_or("Id not provided")?;
        let entry = entry.ok_or("Entry not provided")?;
        
        statement.bind_iter::<_, (_, sqlite::Value)>([
            (":title", entry.title().into()),
            (":kind", entry.kind().to_int().into()),
            (":status", entry.status().to_int().into()),
            (":progress", entry.progress().into()),
            (":good", entry.good().to_int().into()),
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

impl FromRow for (SeriesId, SeriesEntry) {
    
    fn from_row(row: sqlite::Row) -> Option<(SeriesId, SeriesEntry)> {
        let id = SeriesId::from(row.try_read::<i64, _>("id").ok()?);
        let entry = SeriesEntry::new()
            .with_title(row.try_read::<&str, _>("title").ok()?.to_string())
            .with_kind(KindsId::try_from(row.try_read::<i64, _>("kind").ok()?).ok()?)
            .with_status(SeriesStatus::try_from(row.try_read::<i64, _>("status").ok()?).ok()?)
            .with_progress(row.try_read::<i64, _>("progress").ok()?)
            .with_good(SeriesGood::try_from(row.try_read::<i64, _>("good").ok()?).ok()?);
        
        Some((id, entry))
    }
    
}
