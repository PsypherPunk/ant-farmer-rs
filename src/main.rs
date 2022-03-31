use ant_farmer::AntFarmer;
use sqlparser::dialect::MySqlDialect;

#[allow(unused_variables)]
fn main() {
    let sql = r#"cReAtE tAbLe operators_create_consumers (operator_api_key_id    int(11)    NOT NULL, operator_ip_address_id int(11)   nOt NuLl, create_consumers JSON NuLl, created_date datetime nOt NuLl dEfAuLt CURRENT_TIMESTAMP());"#;

    let ant_farmer = AntFarmer::from(MySqlDialect {});

    println!("{}", ant_farmer.mierenneuke(sql).unwrap());
}
