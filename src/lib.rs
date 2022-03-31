use sqlparser::{
    ast::{ColumnDef, ColumnOption, Statement},
    dialect::Dialect,
    parser::{Parser, ParserError},
};

/// Holds the components of a column definition about which we care for display
/// purposes:
///
/// ```sql
/// CREATE TABLE table_name (
///     NAME   TEXT        NOT NULL           DEFAULT ''
///   , {name} {data_type} {options:nullable} {options:default}
/// )
/// ;
/// ```
struct DisplayColumn {
    name: String,
    data_type: String,
    options: (String, String),
}

impl From<&ColumnDef> for DisplayColumn {
    /// Convert a `ColumnDef` to its component, display-relevant parts.
    fn from(column: &ColumnDef) -> Self {
        Self {
            name: column.name.value.to_owned(),
            data_type: column.data_type.to_string(),
            options: column
                .options
                .iter()
                .fold(("".to_owned(), "".to_owned()), |acc, option| {
                    match option.option {
                        ColumnOption::Null | ColumnOption::NotNull => {
                            (option.option.to_string(), acc.1)
                        }
                        ColumnOption::Default(_) => (acc.0, option.option.to_string()),
                        _ => todo!(),
                    }
                }),
        }
    }
}

/// Our nit-picking engine.
///
/// Maintains the internal `dialect` to be used for parsing the input.
pub struct AntFarmer<T: Dialect> {
    dialect: T,
}

impl<T: Dialect> From<T> for AntFarmer<T> {
    fn from(dialect: T) -> Self {
        Self { dialect }
    }
}

impl<T: Dialect> AntFarmer<T> {
    fn get_column_strings(&self, columns: &[ColumnDef]) -> Vec<DisplayColumn> {
        columns
            .iter()
            .map(|column| column.into())
            .collect::<Vec<_>>()
    }

    fn get_column_max_widths(&self, columns: &[DisplayColumn]) -> (usize, usize, usize, usize) {
        columns
            .iter()
            .map(|column| {
                (
                    column.name.len(),
                    column.data_type.len(),
                    column.options.0.len(),
                    column.options.1.len(),
                )
            })
            .fold((0, 0, 0, 0), |acc, (name, type_, null, default)| {
                (
                    name.max(acc.0),
                    type_.max(acc.1),
                    null.max(acc.2),
                    default.max(acc.3),
                )
            })
    }

    /// Parses the input SQL and outputs our "correctly" formatted version.
    ///
    /// Curretly only `CREATE TABLE` is supported.
    pub fn mierenneuke(&self, sql: &str) -> Result<String, ParserError> {
        let ast = Parser::parse_sql(&self.dialect, sql)?;

        let mut output = String::new();

        for statement in ast.iter() {
            match statement {
                Statement::CreateTable { name, columns, .. } => {
                    output += &format!("CREATE TABLE {} (\n", name);

                    let column_strings = self.get_column_strings(columns);

                    let (name_width, type_width, null_width, default_width) =
                        self.get_column_max_widths(&column_strings);

                    let columns =
                    column_strings
                        .iter()
                        .map(|column| {
                            format!(
                                "{:<name_width$} {:<type_width$} {:>null_width$} {:<default_width$}",
                                column.name, column.data_type, column.options.0, column.options.1,
                            )
                        })
                        .collect::<Vec<_>>()
                        .join("\n  , ");
                    output += &format!("    {}\n)\n;", columns);
                }
                _ => todo!(),
            }
        }

        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use sqlparser::dialect::MySqlDialect;

    use super::*;

    #[test]
    fn test_basic_create_table() {
        let sql = r#"cReAtE tAbLe operators_create_consumers (operator_api_key_id    int(11)    NOT NULL, operator_ip_address_id int(11)   nOt NuLl, create_consumers JSON nOt NuLl, created_date datetime nOt NuLl dEfAuLt CURRENT_TIMESTAMP());"#;
        let ant_farmer = AntFarmer::from(MySqlDialect {});
        let expected = r#"CREATE TABLE operators_create_consumers (
    operator_api_key_id    INT(11)  NOT NULL                            
  , operator_ip_address_id INT(11)  NOT NULL                            
  , create_consumers       JSON     NOT NULL                            
  , created_date           datetime NOT NULL DEFAULT CURRENT_TIMESTAMP()
)
;"#;

        let result = ant_farmer.mierenneuke(sql).unwrap();

        assert_eq!(result, expected);
    }

    #[test]
    fn test_create_table_null() {
        let sql = r#"cReAtE tAbLe operators_create_consumers (operator_api_key_id    int(11)    NOT NULL, operator_ip_address_id int(11)   nOt NuLl, create_consumers JSON NuLl, created_date datetime nOt NuLl dEfAuLt CURRENT_TIMESTAMP());"#;
        let ant_farmer = AntFarmer::from(MySqlDialect {});
        let expected = r#"CREATE TABLE operators_create_consumers (
    operator_api_key_id    INT(11)  NOT NULL                            
  , operator_ip_address_id INT(11)  NOT NULL                            
  , create_consumers       JSON         NULL                            
  , created_date           datetime NOT NULL DEFAULT CURRENT_TIMESTAMP()
)
;"#;

        let result = ant_farmer.mierenneuke(sql).unwrap();

        assert_eq!(result, expected);
    }
}
