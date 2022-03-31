use sqlparser::{ast::ColumnDef, dialect::Dialect, parser::Parser};

pub struct AntFarmer<T: Dialect> {
    dialect: T,
}

impl<T: Dialect> From<T> for AntFarmer<T> {
    fn from(dialect: T) -> Self {
        Self { dialect }
    }
}

impl<T: Dialect> AntFarmer<T> {
    fn get_max_column_name_width(&self, columns: &[ColumnDef]) -> Option<usize> {
        columns.iter().map(|column| column.name.value.len()).max()
    }

    fn get_max_column_type_width(&self, columns: &[ColumnDef]) -> Option<usize> {
        columns
            .iter()
            .map(|column| column.data_type.to_string().len())
            .max()
    }

    fn get_column_options(&self, column: &ColumnDef) -> String {
        column
            .options
            .iter()
            .map(|option| option.to_string())
            .collect::<Vec<_>>()
            .join(" ")
    }

    fn get_max_column_options_width(&self, columns: &[ColumnDef]) -> Option<usize> {
        columns
            .iter()
            .map(|column| self.get_column_options(column).len())
            .max()
    }

    pub fn mierenneuke(&self, sql: &str) -> String {
        let ast = Parser::parse_sql(&self.dialect, sql).unwrap();

        let mut output = String::new();

        for statement in ast.iter() {
            match statement {
                sqlparser::ast::Statement::CreateTable { name, columns, .. } => {
                    println!("CREATE TABLE {} (", name);

                    let column_name_width = self.get_max_column_name_width(columns).unwrap();
                    let column_type_width = self.get_max_column_type_width(columns).unwrap();
                    let column_options_width = self.get_max_column_options_width(columns).unwrap();
                    let columns =
                    columns
                        .iter()
                        .map(|column| {
                            format!(
                            "{:<column_name_width$} {:<column_type_width$} {:<column_options_width$}",
                            column.name.value, column.data_type.to_string(), self.get_column_options(column)
                        )
                        })
                        .collect::<Vec<_>>()
                        .join("\n  , ");
                    output += &format!("    {}\n)\n;", columns);
                }
                _ => todo!(),
            }
        }

        output
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

        let result = ant_farmer.mierenneuke(sql);

        assert_eq!(result, expected);
    }
}
