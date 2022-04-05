use sqlparser::{
    ast::{ColumnDef, ColumnOption, Statement, TableConstraint},
    dialect::Dialect,
    parser::{Parser, ParserError},
};

trait AlignedDisplay {
    fn segments(&self) -> Vec<String>;
}

/// Holds the components of a constraint definition about which we care for
/// display purposes:
///
/// ```sql
/// CREATE TABLE table_name (
///   , CONSTRAINT NAME   FOREIGN KEY       (COLUMN)   REFERENCES TARGET_TABLE   (TARGET_COLUMN)
///   , CONSTRAINT {name} {constraint_type} ({column}) REFERENCES {target_table} ({target_column})
/// )
/// ;
/// ```
impl AlignedDisplay for TableConstraint {
    fn segments(&self) -> Vec<String> {
        match self {
            TableConstraint::Unique {
                name,
                columns,
                is_primary,
            } => {
                vec![
                    format!("CONSTRAINT {}", name.clone().unwrap().to_string()),
                    if *is_primary {
                        "PRIMARY KEY".to_string()
                    } else {
                        "UNIQUE".to_string()
                    },
                    columns
                        .iter()
                        .map(|column| column.to_string())
                        .collect::<Vec<_>>()
                        .join(", "),
                    "".to_string(),
                    "".to_string(),
                    "".to_string(),
                    "".to_string(),
                    "".to_string(),
                ]
            }
            TableConstraint::ForeignKey {
                name,
                columns,
                foreign_table,
                referred_columns,
                on_delete,
                on_update,
            } => {
                vec![
                    format!("CONSTRAINT {}", name.clone().unwrap().to_string()),
                    "FOREIGN KEY".to_string(),
                    columns
                        .iter()
                        .map(|column| column.to_string())
                        .collect::<Vec<_>>()
                        .join(", "),
                    "REFERENCES".to_string(),
                    foreign_table.to_string(),
                    referred_columns
                        .iter()
                        .map(|column| column.to_string())
                        .collect::<Vec<_>>()
                        .join(", "),
                    if let Some(action) = on_delete {
                        format!("ON DELETE {}", action)
                    } else {
                        "".to_string()
                    },
                    if let Some(action) = on_update {
                        format!("ON UPDATE {}", action)
                    } else {
                        "".to_string()
                    },
                ]
            }
            TableConstraint::Check { name, expr } => {
                vec![
                    format!("CONSTRAINT {}", name.clone().unwrap().to_string()),
                    format!("CHECK ({})", expr),
                    "".to_string(),
                    "".to_string(),
                    "".to_string(),
                    "".to_string(),
                    "".to_string(),
                    "".to_string(),
                ]
            }
        }
    }
}

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
impl AlignedDisplay for ColumnDef {
    fn segments(&self) -> Vec<String> {
        let nullable = match self
            .options
            .iter()
            .map(|option| &option.option)
            .find(|option| {
                matches!(option, ColumnOption::Null) || matches!(option, ColumnOption::NotNull)
            }) {
            Some(option) => option.to_string(),
            None => "".to_string(),
        };
        let default = match self
            .options
            .iter()
            .map(|option| &option.option)
            .find(|option| matches!(option, ColumnOption::Default(_)))
        {
            Some(option) => option.to_string(),
            None => "".to_string(),
        };

        vec![
            self.name.to_string(),
            self.data_type.to_string(),
            nullable,
            default,
        ]
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
    /// Parses the input SQL and outputs our "correctly" formatted version.
    ///
    /// Currently only `CREATE TABLE` is supported.
    pub fn mierenneuke(&self, sql: &str) -> Result<String, ParserError> {
        let ast = Parser::parse_sql(&self.dialect, sql)?;

        let mut output = String::new();

        for statement in ast.iter() {
            match statement {
                Statement::CreateTable {
                    name,
                    columns,
                    constraints,
                    ..
                } => {
                    output += &format!("CREATE TABLE {} (\n", name);

                    let columns = columns
                        .iter()
                        .map(|column| column.segments())
                        .collect::<Vec<_>>();

                    let constraints = constraints
                        .iter()
                        .map(|constraint| constraint.segments())
                        .collect::<Vec<_>>();

                    let column_widths = columns.iter().fold((0, 0, 0, 0), |acc, column| {
                        (
                            acc.0.max(column[0].len()),
                            acc.1.max(column[1].len()),
                            acc.2.max(column[2].len()),
                            acc.3.max(column[3].len()),
                        )
                    });
                    let constraint_widths =
                        constraints
                            .iter()
                            .fold((0, 0, 0, 0, 0, 0, 0, 0), |acc, column| {
                                (
                                    acc.0.max(column[0].len()),
                                    acc.1.max(column[1].len()),
                                    acc.2.max(column[2].len()),
                                    acc.3.max(column[3].len()),
                                    acc.4.max(column[4].len()),
                                    acc.5.max(column[5].len()),
                                    acc.6.max(column[6].len()),
                                    acc.7.max(column[7].len()),
                                )
                            });

                    let columns = columns
                        .iter()
                        .map(|column| {
                            format!(
                                "{:<name_width$} {:<type_width$} {:>null_width$} {:<default_width$}",
                                column[0], column[1], column[2], column[3],
                                name_width=column_widths.0,
                                type_width=column_widths.1,
                                null_width=column_widths.2,
                                default_width=column_widths.3,
                            )
                        })
                        .collect::<Vec<_>>()
                        .join("\n  , ");

                    let constraints = constraints
                        .iter()
                        .map(|constraint| {
                            format!(
                                "{:<name_width$} {:<type_width$} {:<columns_width$} {:<three$} {:<four$} {:<five$} {:<six$} {:<seven$}",
                                constraint[0],
                                constraint[1],
                                format!("({})", constraint[2]),
                                constraint[3],
                                constraint[4],
                                if constraint[5].len() > 0 { format!("({})", constraint[5]) } else { "".to_owned() },
                                constraint[6],
                                constraint[7],
                                name_width=constraint_widths.0,
                                type_width=constraint_widths.1,
                                columns_width=constraint_widths.2 + 2,
                                three=constraint_widths.3,
                                four=constraint_widths.4,
                                five=constraint_widths.5 + 2,
                                six=constraint_widths.6,
                                seven=constraint_widths.7,
                            )
                            .trim()
                            .to_owned()
                        })
                        .collect::<Vec<_>>()
                        .join("\n  , ");

                    output += &format!("    {}\n", columns);
                    if constraints.len() > 0 {
                        output += &format!("  , {}\n", constraints);
                    }
                    output += ")\n;";
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

    #[test]
    fn test_create_table_constraints() {
        let sql = r#"cReAtE tAbLe operators_create_consumers (operator_api_key_id    int(11)    NOT NULL, operator_ip_address_id int(11)   nOt NuLl, create_consumers JSON NuLl, created_date datetime nOt NuLl dEfAuLt CURRENT_TIMESTAMP() , CONSTRAINT fk_operators_create_consumers_operator_api_key_id FOREIGN KEY (operator_api_key_id ) REFERENCES api_keys (id) , CONSTRAINT fk_operators_create_consumers_operator_ip_address_id  FOREIGN KEY (operator_ip_address_id ) REFERENCES operator_ip_addresses (id) , CONSTRAINT uq_operator_api_key_id_operator_ip_address_id UNIQUE (operator_api_key_id, operator_ip_address_id));"#;
        let ant_farmer = AntFarmer::from(MySqlDialect {});
        let expected = r#"CREATE TABLE operators_create_consumers (
    operator_api_key_id    INT(11)  NOT NULL                            
  , operator_ip_address_id INT(11)  NOT NULL                            
  , create_consumers       JSON         NULL                            
  , created_date           datetime NOT NULL DEFAULT CURRENT_TIMESTAMP()
  , CONSTRAINT fk_operators_create_consumers_operator_api_key_id    FOREIGN KEY (operator_api_key_id)                         REFERENCES api_keys              (id)
  , CONSTRAINT fk_operators_create_consumers_operator_ip_address_id FOREIGN KEY (operator_ip_address_id)                      REFERENCES operator_ip_addresses (id)
  , CONSTRAINT uq_operator_api_key_id_operator_ip_address_id        UNIQUE      (operator_api_key_id, operator_ip_address_id)
)
;"#;

        let result = ant_farmer.mierenneuke(sql).unwrap();

        assert_eq!(result, expected);
    }
}
