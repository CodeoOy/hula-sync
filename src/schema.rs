table! {
    hubspot_projects (id) {
        id -> Uuid,
        hula_id -> Uuid,
        hubspot_id -> Varchar,
        name -> Varchar,
        updated_by -> Varchar,
    }
}

table! {
    odoo_projects (id) {
        id -> Uuid,
        hula_id -> Uuid,
        odoo_id -> Int4,
        name -> Varchar,
        updated_by -> Varchar,
    }
}

allow_tables_to_appear_in_same_query!(
    hubspot_projects,
    odoo_projects,
);
