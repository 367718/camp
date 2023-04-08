use std::{
    collections::HashSet,
    error::Error,
};

use super::{ Candidates, CandidatesId, CandidatesEntry, CandidatesCurrent, SeriesId };
use crate::{ Queries, Binds, FromRow };

impl Queries for Candidates {
    
    fn create(&self) -> &str {
        "CREATE TABLE IF NOT EXISTS candidates (
            id INTEGER PRIMARY KEY, 
            series INTEGER, 
            title TEXT, 
            grp TEXT, 
            quality TEXT, 
            offset INTEGER, 
            current INTEGER, 
            downloaded TEXT
        );"
    }
    
    fn count(&self) -> &str {
        "SELECT COUNT(*) FROM candidates;"
    }
    
    fn select(&self) -> &str {
        "SELECT 
        id, 
        series, 
        title, 
        grp, 
        quality, 
        offset, 
        current, 
        downloaded 
        FROM candidates"
    }
    
    fn insert(&self) -> &str {
        "INSERT INTO candidates 
        (series, title, grp, quality, offset, current, downloaded) 
        VALUES 
        (:series, :title, :grp, :quality, :offset, :current, :downloaded) 
        RETURNING id;"
    }
    
    fn update(&self) -> &str {
        "UPDATE candidates SET 
        series = :series, 
        title = :title, 
        grp = :grp, 
        quality = :quality, 
        offset = :offset, 
        current = :current, 
        downloaded = :downloaded 
        WHERE id = :id;"
    }
    
    fn delete(&self) -> &str {
        "DELETE FROM candidates WHERE id = :id;"
    }
    
}

impl Binds for (Option<CandidatesId>, Option<&CandidatesEntry>) {
    
    fn insert(self, statement: &mut sqlite::Statement) -> Result<(), Box<dyn Error>> {
        let (_, entry) = self;
        
        let entry = entry.ok_or("Entry not provided")?;
        
        statement.bind_iter::<_, (_, sqlite::Value)>([
            (":series", entry.series().to_int().into()),
            (":title", entry.title().into()),
            (":grp", entry.group().into()),
            (":quality", entry.quality().into()),
            (":offset", entry.offset().into()),
            (":current", entry.current().to_int().into()),
            (":downloaded", encode_downloaded(entry.downloaded()).into()),
        ])?;
        
        Ok(())
    }
    
    fn update(self, statement: &mut sqlite::Statement) -> Result<(), Box<dyn Error>> {
        let (id, entry) = self;
        
        let id = id.ok_or("Id not provided")?;
        let entry = entry.ok_or("Entry not provided")?;
        
        statement.bind_iter::<_, (_, sqlite::Value)>([
            (":series", entry.series().to_int().into()),
            (":title", entry.title().into()),
            (":grp", entry.group().into()),
            (":quality", entry.quality().into()),
            (":offset", entry.offset().into()),
            (":current", entry.current().to_int().into()),
            (":downloaded", encode_downloaded(entry.downloaded()).into()),
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

impl FromRow for (CandidatesId, CandidatesEntry) {
    
    fn from_row(row: sqlite::Row) -> Option<(CandidatesId, CandidatesEntry)> {
        let id = CandidatesId::from(row.try_read::<i64, _>("id").ok()?);
        let entry = CandidatesEntry::new()
            .with_series(SeriesId::try_from(row.try_read::<i64, _>("series").ok()?).ok()?)
            .with_title(row.try_read::<&str, _>("title").ok()?.to_string())
            .with_group(row.try_read::<&str, _>("grp").ok()?.to_string())
            .with_quality(row.try_read::<&str, _>("quality").ok()?.to_string())
            .with_offset(row.try_read::<i64, _>("offset").ok()?)
            .with_current(CandidatesCurrent::try_from(row.try_read::<i64, _>("current").ok()?).ok()?)
            .with_downloaded(decode_downloaded(row.try_read::<&str, _>("downloaded").ok()?));
        
        Some((id, entry))
    }
    
}

fn decode_downloaded(value: &str) -> HashSet<i64> {
    value.split(',')
        .flat_map(str::parse::<i64>)
        .collect()
}

fn encode_downloaded(value: &HashSet<i64>) -> String {
    value.iter()
        .map(ToString::to_string)
        .collect::<Vec<String>>()
        .join(",")
}
