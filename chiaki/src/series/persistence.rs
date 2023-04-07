use std::error::Error;

use super::{ Series, SeriesId, SeriesEntry, SeriesStatus, SeriesGood, KindsId };
use crate::{ PersistenceQueries, PersistenceBinds, FromRow };

impl PersistenceQueries for Series {
    
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

impl PersistenceBinds for SeriesId {
    
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

impl PersistenceBinds for SeriesEntry {
    
    fn insert(&self, statement: &mut sqlite::Statement) -> Result<(), Box<dyn Error>> {
        statement.bind((":title", self.title()))?;
        statement.bind((":kind", self.kind().as_int()))?;
        statement.bind((":status", self.status().as_int()))?;
        statement.bind((":progress", self.progress()))?;
        statement.bind((":good", self.good().as_int()))?;
        
        Ok(())
    }
    
    fn update(&self, statement: &mut sqlite::Statement) -> Result<(), Box<dyn Error>> {
        statement.bind_iter::<_, (_, sqlite::Value)>([
            (":title", self.title().into()),
            (":kind", self.kind().as_int().into()),
            (":status", self.status().as_int().into()),
            (":progress", self.progress().into()),
            (":good", self.good().as_int().into()),
        ])?;
        
        Ok(())
    }
    
    fn delete(&self, _statement: &mut sqlite::Statement) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
    
}

impl FromRow for SeriesId {
    
    fn from_row(row: &sqlite::Row) -> Option<Self> {
        Some(Self::from(row.try_read::<i64, _>("id").ok()?))
    }
    
}

impl FromRow for SeriesEntry {
    
    fn from_row(row: &sqlite::Row) -> Option<Self> {
        let entry = Self::new()
            .with_title(row.try_read::<&str, _>("title").ok()?.to_string())
            .with_kind(KindsId::try_from(row.try_read::<i64, _>("kind").ok()?).ok()?)
            .with_status(SeriesStatus::try_from(row.try_read::<i64, _>("status").ok()?).ok()?)
            .with_progress(row.try_read::<i64, _>("progress").ok()?)
            .with_good(SeriesGood::try_from(row.try_read::<i64, _>("good").ok()?).ok()?);
        
        Some(entry)
    }
    
}
