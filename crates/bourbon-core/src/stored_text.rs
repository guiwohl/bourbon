macro_rules! stored_text_enum {
    ($name:ident) => {
        impl rusqlite::types::ToSql for $name {
            fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
                self.as_stored().to_sql()
            }
        }

        impl rusqlite::types::FromSql for $name {
            fn column_result(
                value: rusqlite::types::ValueRef<'_>,
            ) -> rusqlite::types::FromSqlResult<Self> {
                Self::from_stored(value.as_str()?).ok_or(rusqlite::types::FromSqlError::InvalidType)
            }
        }
    };
}

pub(crate) use stored_text_enum;
