table! {
    odoo_projects (id) {
        id -> Uuid,
        hula_id -> Uuid,
        odoo_id -> Int4,
        name -> Varchar,
        updated_by -> Varchar,
    }
}
