table! {
    use diesel::sql_types::*;
    use crate::enum_types::*;

    admins (id) {
        id -> Int8,
        name -> Varchar,
        surname -> Varchar,
        patronymic -> Varchar,
        phone -> Varchar,
        pass_hash -> Varchar,
        is_deleted -> Bool,
        picture -> Nullable<Varchar>,
        creation_datetime -> Timestamp,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::enum_types::*;

    auth (id) {
        id -> Int8,
        login -> Varchar,
        auth_type -> Varchar,
        roles -> Array<Text>,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::enum_types::*;

    courier_rating (id) {
        id -> Int8,
        courier_id -> Int8,
        order_id -> Int8,
        look -> Int2,
        politeness -> Int2,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::enum_types::*;

    couriers (id) {
        id -> Int8,
        name -> Varchar,
        surname -> Varchar,
        patronymic -> Varchar,
        phone -> Varchar,
        pass_hash -> Varchar,
        is_blocked -> Bool,
        is_warned -> Bool,
        is_deleted -> Bool,
        is_in_order -> Bool,
        current_rate_amount -> Int8,
        current_rate_count -> Int8,
        picture -> Nullable<Varchar>,
        cash -> Int8,
        term -> Int8,
        salary -> Int8,
        creation_datetime -> Timestamp,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::enum_types::*;

    couriers_approvals (id) {
        id -> Int8,
        courier_id -> Int8,
        order_id -> Int8,
        datetime -> Timestamp,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::enum_types::*;

    couriers_for_curators_xls_reports (id) {
        id -> Int8,
        filename -> Varchar,
        creation_date -> Date,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::enum_types::*;

    couriers_xls_reports (id) {
        id -> Int8,
        courier_id -> Int8,
        filename -> Varchar,
        creation_date -> Date,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::enum_types::*;

    curators (id) {
        id -> Int8,
        name -> Varchar,
        surname -> Varchar,
        patronymic -> Varchar,
        phone -> Varchar,
        pass_hash -> Varchar,
        is_deleted -> Bool,
        picture -> Nullable<Varchar>,
        creation_datetime -> Timestamp,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::enum_types::*;

    notifications (id) {
        id -> Int8,
        title -> Varchar,
        message -> Varchar,
        creation_datetime -> Timestamp,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::enum_types::*;

    notifications_to_couriers (id) {
        id -> Int8,
        courier_id -> Int8,
        notific_id -> Int8,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::enum_types::*;

    orders (id) {
        id -> Int8,
        restaurant_id -> Nullable<Int8>,
        session_id -> Nullable<Int8>,
        details -> Varchar,
        is_big_order -> Bool,
        delivery_address -> Varchar,
        address_lat -> Float8,
        address_lng -> Float8,
        method -> Paymethod,
        courier_share -> Int8,
        order_price -> Int8,
        cooking_time -> Time,
        client_phone -> Varchar,
        client_comment -> Varchar,
        status -> Orderstatus,
        finalize_comment -> Nullable<Varchar>,
        finalize_datetime -> Nullable<Timestamp>,
        take_datetime -> Nullable<Timestamp>,
        delivery_datetime -> Nullable<Timestamp>,
        creation_datetime -> Timestamp,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::enum_types::*;

    pending_files (id) {
        id -> Int8,
        url -> Varchar,
        upload -> Timestamp,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::enum_types::*;

    restaurants (id) {
        id -> Int8,
        name -> Varchar,
        address -> Varchar,
        phone -> Varchar,
        pass_hash -> Varchar,
        location_lat -> Float8,
        location_lng -> Float8,
        working_from -> Array<Time>,
        working_till -> Array<Time>,
        is_working -> Bool,
        is_deleted -> Bool,
        creation_datetime -> Timestamp,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::enum_types::*;

    restaurants_for_curators_xls_reports (id) {
        id -> Int8,
        filename -> Varchar,
        creation_date -> Date,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::enum_types::*;

    restaurants_xls_reports (id) {
        id -> Int8,
        restaurant_id -> Int8,
        filename -> Varchar,
        creation_date -> Date,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::enum_types::*;

    sessions (id) {
        id -> Int8,
        courier_id -> Int8,
        start_time -> Time,
        end_time -> Time,
        session_day -> Date,
        end_real_time -> Nullable<Time>,
        has_terminal -> Bool,
        transport -> Transporttype,
    }
}

joinable!(courier_rating -> couriers (courier_id));
joinable!(courier_rating -> orders (order_id));
joinable!(couriers_approvals -> couriers (courier_id));
joinable!(couriers_approvals -> orders (order_id));
joinable!(couriers_xls_reports -> couriers (courier_id));
joinable!(orders -> restaurants (restaurant_id));
joinable!(orders -> sessions (session_id));
joinable!(restaurants_xls_reports -> restaurants (restaurant_id));
joinable!(sessions -> couriers (courier_id));

allow_tables_to_appear_in_same_query!(
    admins,
    auth,
    courier_rating,
    couriers,
    couriers_approvals,
    couriers_for_curators_xls_reports,
    couriers_xls_reports,
    curators,
    notifications,
    notifications_to_couriers,
    orders,
    pending_files,
    restaurants,
    restaurants_for_curators_xls_reports,
    restaurants_xls_reports,
    sessions,
);
