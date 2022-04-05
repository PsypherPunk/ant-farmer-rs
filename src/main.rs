use ant_farmer::AntFarmer;
use sqlparser::dialect::MySqlDialect;

#[allow(unused_variables)]
fn main() {
    let sql = r#"cReAtE tAbLe operators_create_consumers (operator_api_key_id    int(11)    NOT NULL, operator_ip_address_id int(11)   nOt NuLl, create_consumers JSON NuLl, created_date datetime nOt NuLl dEfAuLt CURRENT_TIMESTAMP() , CONSTRAINT fk_operators_create_consumers_operator_api_key_id FOREIGN KEY (operator_api_key_id ) REFERENCES api_keys (id) , CONSTRAINT fk_operators_create_consumers_operator_ip_address_id  FOREIGN KEY (operator_ip_address_id ) REFERENCES operator_ip_addresses (id), CONSTRAINT uq_operator_api_key_id_operator_ip_address_id UNIQUE (operator_api_key_id, operator_ip_address_id));"#;

    let ant_farmer = AntFarmer::from(MySqlDialect {});

    println!("{}", ant_farmer.mierenneuke(sql).unwrap());
}
