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

table! {
	hula_call_log (id) {
		id -> Uuid,
		hula_id -> Nullable<Uuid>,
		odoo_id -> Int4,
		url -> Varchar,
		verb -> Varchar,
		payload -> Varchar,
		status -> Int4,
		response -> Varchar,
		updated_by -> Varchar,
		updated_at -> Timestamp,
	}
}

table! {
	odoo_call_log (id) {
		id -> Uuid,
		script -> Varchar,
		param1 -> Nullable<Varchar>,
		param2 -> Nullable<Varchar>,
		param3 -> Nullable<Varchar>,
		param4 -> Nullable<Varchar>,
		param5 -> Nullable<Varchar>,
		param6 -> Nullable<Varchar>,
		ok -> Bool,
		response -> Nullable<Varchar>,
		updated_by -> Varchar,
		updated_at -> Timestamp,
	}
}

allow_tables_to_appear_in_same_query!(hubspot_projects, odoo_projects,);
