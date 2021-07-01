table! {
    admin (id) {
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
    courier_rating (id) {
        id -> Int8,
        courier_id -> Int8,
        order_id -> Int8,
        look -> Int2,
        politeness -> Int2,
    }
}

table! {
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
        current_rate -> Nullable<Int2>,
        picture -> Nullable<Varchar>,
        cash -> Int8,
        term -> Int8,
        salary -> Int8,
        creation_datetime -> Timestamp,
    }
}

table! {
    curator (id) {
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
    orders (id) {
        id -> Int8,
        restaurant_id -> Int8,
        courier_id -> Int8,
        details -> Varchar,
        is_big_order -> Bool,
        delivery_address -> Varchar,
        address_lat -> Int8,
        address_lng -> Int8,
        method -> Pay_method,
        courier_share -> Int8,
        order_price -> Int8,
        cooking_time -> Interval,
        client_phone -> Varchar,
        client_comment -> Varchar,
        status -> Order_status,
        finalize_comment -> Nullable<Varchar>,
        take_datetime -> Nullable<Timestamp>,
        creation_datetime -> Timestamp,
    }
}

table! {
    orders_history (id) {
        id -> Int8,
        restaurant_id -> Int8,
        courier_id -> Int8,
        details -> Varchar,
        is_big_order -> Bool,
        delivery_address -> Varchar,
        address_lat -> Int8,
        address_lng -> Int8,
        method -> Pay_method,
        courier_share -> Int8,
        order_price -> Int8,
        cooking_time -> Time,
        client_phone -> Varchar,
        client_comment -> Varchar,
        status -> Order_status,
        finalize_comment -> Nullable<Varchar>,
        take_datetime -> Nullable<Timestamp>,
        creation_datetime -> Timestamp,
    }
}

table! {
    restaurants (id) {
        id -> Int8,
        name -> Varchar,
        address -> Varchar,
        phone -> Varchar,
        pass_hash -> Varchar,
        location_lat -> Int8,
        location_lng -> Int8,
        working_from -> Array<Time>,
        working_till -> Array<Time>,
        is_working -> Bool,
        is_deleted -> Bool,
        creation_datetime -> Timestamp,
    }
}

table! {
    sessions (id) {
        id -> Int8,
        courier_id -> Int8,
        start_datetime -> Timestamp,
        end_datetime -> Timestamp,
        session_day -> Date,
        transport -> Transport_type,
    }
}

table! {
    sessions_history (id) {
        id -> Int8,
        courier_id -> Int8,
        start_datetime -> Timestamp,
        end_datetime -> Timestamp,
        session_day -> Date,
        transport -> Transport_type,
    }
}

joinable!(orders -> couriers (courier_id));
joinable!(orders -> restaurants (restaurant_id));
joinable!(orders_history -> restaurants (restaurant_id));

allow_tables_to_appear_in_same_query!(
    admin,
    courier_rating,
    couriers,
    curator,
    orders,
    orders_history,
    restaurants,
    sessions,
    sessions_history,
);
