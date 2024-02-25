use std::collections::HashMap;
use std::fmt::Display;

use sqlparser::ast::{ColumnOption, Statement, TableConstraint};
use sqlparser::dialect::GenericDialect;
use sqlparser::parser::Parser;

#[derive(Clone, Debug, PartialEq)]
pub enum QueryAst {
    Parsed(Vec<Statement>),
    InvalidSQL(String),
}

impl Display for QueryAst {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QueryAst::Parsed(stmt) => {
                let res = stmt
                    .into_iter()
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>()
                    .join("\n");
                write!(f, "{}", res)
            }
            QueryAst::InvalidSQL(_) => write!(
                f,
                "Invalid SQL Statement. Only Create Table statement is supported."
            ),
        }
    }
}

pub fn parse_query(raw_str: &str) -> QueryAst {
    let dialect = GenericDialect {};
    let parsed_query = Parser::parse_sql(&dialect, raw_str);
    match parsed_query {
        Ok(sql) => QueryAst::Parsed(sql),
        Err(_) => QueryAst::InvalidSQL("Invalid SQL statement".to_string()),
    }
}

pub fn ast_as_d2_spec(ast: Statement) -> ParsedTable {
    match ast {
        Statement::CreateTable {
            or_replace: _,
            temporary: _,
            external: _,
            global: _,
            if_not_exists: _,
            transient: _,
            name,
            columns,
            constraints,
            hive_distribution: _,
            hive_formats: _,
            table_properties: _,
            with_options: _,
            file_format: _,
            location: _,
            query: _,
            without_rowid: _,
            like: _,
            clone: _,
            engine: _,
            comment: _,
            auto_increment_offset: _,
            default_charset: _,
            collation: _,
            on_commit: _,
            on_cluster: _,
            order_by: _,
            partition_by: _,
            cluster_by: _,
            options: _,
            strict: _,
        } => {
            let table_name = name.0.first().expect("Table name cannot be empty");
            let all_columns = columns
                .into_iter()
                .map(|col| {
                    let col_name = col.name.value;
                    let data_type = col.data_type.to_string();
                    let col_constraints = col
                        .options
                        .into_iter()
                        .map(|opt| match opt.option {
                            ColumnOption::Null => Constraint::Null,
                            ColumnOption::NotNull => Constraint::NotNull,
                            ColumnOption::Unique {
                                is_primary,
                                characteristics: _,
                            } => {
                                if is_primary {
                                    Constraint::PrimaryKey
                                } else {
                                    Constraint::Unique
                                }
                            }
                            ColumnOption::ForeignKey {
                                foreign_table: _,
                                referred_columns: _,
                                on_delete: _,
                                on_update: _,
                                characteristics: _,
                            } => Constraint::ForeignKey,
                            _ => Constraint::Undefined,
                        })
                        .collect::<Vec<_>>();
                    ColumnDefinition {
                        name: col_name,
                        data_type,
                        constraint: col_constraints,
                    }
                })
                .collect::<Vec<_>>();
            let relationships = constraints
                .clone()
                .into_iter()
                .map(|constraint| table_relationship(table_name.clone().value, constraint))
                .collect::<Vec<HashMap<ParentTable, ReferencedTable>>>()
                .into_iter()
                .flatten()
                .collect::<HashMap<ParentTable, ReferencedTable>>();
            ParsedTable::D2Table(D2Table {
                table_name: table_name.clone().value,
                columns: all_columns,
                relationships,
            })
        }
        _ => ParsedTable::NotSupported,
    }
}

pub fn table_relationship(
    parent_table: String,
    constraint: TableConstraint,
) -> HashMap<ParentTable, ReferencedTable> {
    match constraint {
        TableConstraint::ForeignKey {
            name: _,
            columns,
            foreign_table,
            referred_columns,
            on_delete: _,
            on_update: _,
            characteristics: _,
        } => {
            let fk_table = foreign_table.0.first().unwrap();
            let parent_cols = columns
                .into_iter()
                .map(|col| ParentTable {
                    table: parent_table.clone(),
                    column: col.value,
                })
                .collect::<Vec<ParentTable>>();
            let ref_cols = referred_columns
                .into_iter()
                .map(|col| ReferencedTable {
                    foreign_table: fk_table.clone().value,
                    referred_column: col.value,
                })
                .collect::<Vec<ReferencedTable>>();
            let references = parent_cols.into_iter().zip(ref_cols.into_iter());
            let column_references = references
                .map(move |(pr, rf)| (pr, rf))
                .collect::<HashMap<_, _>>();
            column_references
        }
        _ => HashMap::new(),
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ParsedTable {
    D2Table(D2Table),
    NotSupported,
}

impl Display for ParsedTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParsedTable::D2Table(d2_table) => write!(f, "{}", d2_table.to_string())?,
            ParsedTable::NotSupported => write!(
                f,
                "Unsupported SQL statement. Only Created Table statement is supported."
            )?,
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct D2Table {
    pub table_name: String,
    pub columns: Vec<ColumnDefinition>,
    pub relationships: HashMap<ParentTable, ReferencedTable>,
}

impl Display for D2Table {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}: {{", self.table_name)?;
        writeln!(f, "\t{}", format!("shape: sql_table"))?;
        let col_def = self
            .columns
            .clone()
            .into_iter()
            .map(|col| {
                let col_constraints = col
                    .constraint
                    .into_iter()
                    .map(|x| x.to_string())
                    .filter(|x| !x.is_empty())
                    .collect::<Vec<_>>()
                    .join(", ");
                let col_with_type = format!("\t{}: {}", col.name, col.data_type);
                if col_constraints.is_empty() {
                    col_with_type
                } else {
                    format!("{} {{constraint: {}}}", col_with_type, col_constraints)
                }
            })
            .collect::<Vec<_>>()
            .join("\n");
        writeln!(f, "{}", col_def)?;
        writeln!(f, "}}")?;

        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ColumnDefinition {
    pub name: String,
    pub data_type: String,
    pub constraint: Vec<Constraint>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Constraint {
    Null,
    NotNull,
    PrimaryKey,
    ForeignKey,
    Unique,
    Undefined,
}
impl Display for Constraint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Constraint::Null => write!(f, "NULL"),
            Constraint::NotNull => write!(f, "NOT NULL"),
            Constraint::PrimaryKey => write!(f, "PK"),
            Constraint::ForeignKey => write!(f, "FK"),
            Constraint::Unique => write!(f, "UNIQUE"),
            Constraint::Undefined => write!(f, ""),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ForeignKeyInfo {
    pub foreign_table: String,
    pub referred_columns: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ParentTable {
    pub table: String,
    pub column: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ReferencedTable {
    pub foreign_table: String,
    pub referred_column: String,
}
