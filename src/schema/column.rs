//! [`ColumnDef`] — SQL column definition with fluent modifiers.

/// Referential integrity action for foreign key constraints.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ForeignAction {
    Cascade,
    Restrict,
    SetNull,
    SetDefault,
    NoAction,
}

impl ForeignAction {
    pub fn to_sql(&self) -> &'static str {
        match self {
            Self::Cascade => "CASCADE",
            Self::Restrict => "RESTRICT",
            Self::SetNull => "SET NULL",
            Self::SetDefault => "SET DEFAULT",
            Self::NoAction => "NO ACTION",
        }
    }
}

/// A foreign key constraint definition.
#[derive(Debug, Clone)]
pub struct ForeignKey {
    pub column: String,
    pub references_table: String,
    pub references_column: String,
    pub on_delete: ForeignAction,
    pub on_update: ForeignAction,
}

impl ForeignKey {
    pub fn new(column: impl Into<String>) -> Self {
        Self {
            column: column.into(),
            references_table: String::new(),
            references_column: "id".to_string(),
            on_delete: ForeignAction::Restrict,
            on_update: ForeignAction::Restrict,
        }
    }

    pub fn references(mut self, table: impl Into<String>, column: impl Into<String>) -> Self {
        self.references_table = table.into();
        self.references_column = column.into();
        self
    }

    pub fn on_delete(mut self, action: ForeignAction) -> Self {
        self.on_delete = action;
        self
    }

    pub fn on_update(mut self, action: ForeignAction) -> Self {
        self.on_update = action;
        self
    }

    pub fn to_sql(&self) -> String {
        format!(
            "FOREIGN KEY ({}) REFERENCES {} ({}) ON DELETE {} ON UPDATE {}",
            self.column,
            self.references_table,
            self.references_column,
            self.on_delete.to_sql(),
            self.on_update.to_sql(),
        )
    }
}

/// An index definition.
#[derive(Debug, Clone)]
pub struct IndexDef {
    pub columns: Vec<String>,
    pub unique: bool,
    pub name: Option<String>,
}

impl IndexDef {
    pub fn new(columns: Vec<String>, unique: bool) -> Self {
        Self { columns, unique, name: None }
    }

    pub fn named(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn index_name(&self, table: &str) -> String {
        self.name.clone().unwrap_or_else(|| {
            let suffix = if self.unique { "unique" } else { "idx" };
            format!("{}_{}_{}", table, self.columns.join("_"), suffix)
        })
    }
}

/// Raw SQL dialect used for DDL generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SchemaDialect {
    #[default]
    Postgres,
    Sqlite,
    Mysql,
}

/// The SQL data type for a column.
#[derive(Debug, Clone)]
pub enum ColumnType {
    Increments,
    BigIncrements,
    Uuid,
    String(u32),
    Text,
    Integer,
    BigInteger,
    SmallInteger,
    Float,
    Double,
    Decimal(u32, u32),
    Boolean,
    Date,
    DateTime,
    Timestamp,
    Json,
    Binary,
    Enum(Vec<String>),
    /// Arbitrary raw SQL type string.
    Raw(String),
}

impl ColumnType {
    pub fn to_sql(&self, dialect: SchemaDialect) -> String {
        match self {
            Self::Increments => match dialect {
                SchemaDialect::Postgres => "SERIAL".to_string(),
                SchemaDialect::Sqlite => "INTEGER".to_string(),
                SchemaDialect::Mysql => "INT AUTO_INCREMENT".to_string(),
            },
            Self::BigIncrements => match dialect {
                SchemaDialect::Postgres => "BIGSERIAL".to_string(),
                SchemaDialect::Sqlite => "INTEGER".to_string(),
                SchemaDialect::Mysql => "BIGINT AUTO_INCREMENT".to_string(),
            },
            Self::Uuid => match dialect {
                SchemaDialect::Postgres => "UUID".to_string(),
                SchemaDialect::Sqlite => "TEXT".to_string(),
                SchemaDialect::Mysql => "CHAR(36)".to_string(),
            },
            Self::String(len) => match dialect {
                SchemaDialect::Mysql => format!("VARCHAR({len})"),
                _ => format!("VARCHAR({len})"),
            },
            Self::Text => "TEXT".to_string(),
            Self::Integer => "INTEGER".to_string(),
            Self::BigInteger => match dialect {
                SchemaDialect::Sqlite => "INTEGER".to_string(),
                _ => "BIGINT".to_string(),
            },
            Self::SmallInteger => match dialect {
                SchemaDialect::Sqlite => "INTEGER".to_string(),
                _ => "SMALLINT".to_string(),
            },
            Self::Float => match dialect {
                SchemaDialect::Postgres => "REAL".to_string(),
                SchemaDialect::Sqlite => "REAL".to_string(),
                SchemaDialect::Mysql => "FLOAT".to_string(),
            },
            Self::Double => match dialect {
                SchemaDialect::Postgres => "DOUBLE PRECISION".to_string(),
                SchemaDialect::Sqlite => "REAL".to_string(),
                SchemaDialect::Mysql => "DOUBLE".to_string(),
            },
            Self::Decimal(precision, scale) => match dialect {
                SchemaDialect::Sqlite => "NUMERIC".to_string(),
                _ => format!("DECIMAL({precision},{scale})"),
            },
            Self::Boolean => match dialect {
                SchemaDialect::Sqlite => "INTEGER".to_string(),
                SchemaDialect::Mysql => "TINYINT(1)".to_string(),
                SchemaDialect::Postgres => "BOOLEAN".to_string(),
            },
            Self::Date => match dialect {
                SchemaDialect::Sqlite => "TEXT".to_string(),
                _ => "DATE".to_string(),
            },
            Self::DateTime => match dialect {
                SchemaDialect::Postgres => "TIMESTAMPTZ".to_string(),
                SchemaDialect::Sqlite => "TEXT".to_string(),
                SchemaDialect::Mysql => "DATETIME".to_string(),
            },
            Self::Timestamp => match dialect {
                SchemaDialect::Postgres => "TIMESTAMPTZ".to_string(),
                SchemaDialect::Sqlite => "TEXT".to_string(),
                SchemaDialect::Mysql => "TIMESTAMP".to_string(),
            },
            Self::Json => match dialect {
                SchemaDialect::Postgres => "JSONB".to_string(),
                SchemaDialect::Mysql => "JSON".to_string(),
                SchemaDialect::Sqlite => "TEXT".to_string(),
            },
            Self::Binary => match dialect {
                SchemaDialect::Postgres => "BYTEA".to_string(),
                SchemaDialect::Sqlite => "BLOB".to_string(),
                SchemaDialect::Mysql => "BLOB".to_string(),
            },
            Self::Enum(values) => match dialect {
                SchemaDialect::Postgres | SchemaDialect::Mysql => {
                    let vals = values.iter().map(|v| format!("'{v}'")).collect::<Vec<_>>().join(", ");
                    format!("VARCHAR(255) CHECK (value IN ({vals}))")
                }
                SchemaDialect::Sqlite => "TEXT".to_string(),
            },
            Self::Raw(s) => s.clone(),
        }
    }
}

/// A single column definition with modifiers.
#[derive(Debug, Clone)]
pub struct ColumnDef {
    pub name: String,
    pub col_type: ColumnType,
    pub nullable: bool,
    pub default: Option<String>,
    pub unique: bool,
    pub primary: bool,
}

impl ColumnDef {
    pub fn new(name: impl Into<String>, col_type: ColumnType) -> Self {
        Self {
            name: name.into(),
            col_type,
            nullable: false,
            default: None,
            unique: false,
            primary: false,
        }
    }

    pub fn nullable(mut self) -> Self {
        self.nullable = true;
        self
    }

    pub fn not_null(mut self) -> Self {
        self.nullable = false;
        self
    }

    pub fn default(mut self, val: impl Into<String>) -> Self {
        self.default = Some(val.into());
        self
    }

    pub fn unique(mut self) -> Self {
        self.unique = true;
        self
    }

    pub fn primary(mut self) -> Self {
        self.primary = true;
        self
    }

    /// Generate the SQL fragment for this column (without a trailing comma).
    pub fn to_sql(&self, dialect: SchemaDialect) -> String {
        let type_sql = self.col_type.to_sql(dialect);
        let mut parts = vec![format!("{} {}", self.name, type_sql)];

        if self.primary {
            parts.push("PRIMARY KEY".to_string());
        }
        if !self.nullable {
            // Auto-increment types imply NOT NULL in most dialects
            match &self.col_type {
                ColumnType::Increments | ColumnType::BigIncrements => {}
                _ => parts.push("NOT NULL".to_string()),
            }
        } else {
            parts.push("NULL".to_string());
        }
        if self.unique {
            parts.push("UNIQUE".to_string());
        }
        if let Some(ref default) = self.default {
            parts.push(format!("DEFAULT {default}"));
        }

        parts.join(" ")
    }
}
