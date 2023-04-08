use std::{
    collections::HashSet,
    error::Error,
};

use super::{ Candidates, CandidatesId, CandidatesEntry, CandidatesCurrent, SeriesId };
use crate::{ PersistenceQueries, PersistenceBinds, FromRow };

impl PersistenceQueries for Candidates {
    
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

impl PersistenceBinds for CandidatesId {
    
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

impl PersistenceBinds for CandidatesEntry {
    
    fn insert(&self, statement: &mut sqlite::Statement) -> Result<(), Box<dyn Error>> {
        statement.bind((":series", i64::from(self.series())))?;
        statement.bind((":title", self.title()))?;
        statement.bind((":grp", self.group()))?;
        statement.bind((":quality", self.quality()))?;
        statement.bind((":offset", self.offset()))?;
        statement.bind((":current", i64::from(self.current())))?;
        statement.bind((":downloaded", encode_downloaded(self.downloaded()).as_str()))?;
        
        Ok(())
    }
    
    fn update(&self, statement: &mut sqlite::Statement) -> Result<(), Box<dyn Error>> {
        statement.bind_iter::<_, (_, sqlite::Value)>([
            (":series", i64::from(self.series()).into()),
            (":title", self.title().into()),
            (":grp", self.group().into()),
            (":quality", self.quality().into()),
            (":offset", self.offset().into()),
            (":current", i64::from(self.current()).into()),
            (":downloaded", encode_downloaded(self.downloaded()).into()),
        ])?;
        
        Ok(())
    }
    
    fn delete(&self, _statement: &mut sqlite::Statement) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
    
}

impl FromRow for CandidatesId {
    
    fn from_row(row: &sqlite::Row) -> Option<Self> {
        Some(Self::from(row.try_read::<i64, _>("id").ok()?))
    }
    
}

impl FromRow for CandidatesEntry {
    
    fn from_row(row: &sqlite::Row) -> Option<Self> {
        let entry = Self::new()
            .with_series(SeriesId::try_from(row.try_read::<i64, _>("series").ok()?).ok()?)
            .with_title(row.try_read::<&str, _>("title").ok()?.to_string())
            .with_group(row.try_read::<&str, _>("grp").ok()?.to_string())
            .with_quality(row.try_read::<&str, _>("quality").ok()?.to_string())
            .with_offset(row.try_read::<i64, _>("offset").ok()?)
            .with_current(CandidatesCurrent::try_from(row.try_read::<i64, _>("current").ok()?).ok()?)
            .with_downloaded(decode_downloaded(row.try_read::<&str, _>("downloaded").ok()?));
        
        Some(entry)
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
