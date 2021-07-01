DROP TABLE curator;
DROP TABLE couriers;
DROP TABLE admin;
DROP TABLE restaurants;
DROP TABLE orders;
DROP TABLE sessions_history;
DROP TABLE couriers_approvals;
DROP TABLE orders_history;
DROP TABLE courier_rating;
DROP TYPE PayMethod cascade;
DROP TYPE TransportType cascade;
DROP TYPE OrderStatus cascade;


CREATE TABLE curator (
    id                  BIGSERIAL   PRIMARY KEY,
    name                VARCHAR     NOT NULL,
    surname             VARCHAR     NOT NULL,
    patronymic          VARCHAR     NOT NULL,
    phone               VARCHAR     NOT NULL UNIQUE,
    pass_hash           VARCHAR     NOT NULL,
    is_deleted          BOOL        NOT NULL DEFAULT FALSE,
    picture             VARCHAR,
    creation_datetime   TIMESTAMP   NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE couriers_to_curators (
    id                  BIGSERIAL   PRIMARY KEY,
    courier_id          BIGINT      REFERENCES couriers(id),
    curator_id          BIGINT      REFERENCES curator(id)
);

CREATE TABLE couriers (
    id                  BIGSERIAL   PRIMARY KEY,
    name                VARCHAR     NOT NULL,
    surname             VARCHAR     NOT NULL,
    patronymic          VARCHAR     NOT NULL,
    phone               VARCHAR     NOT NULL UNIQUE,
    pass_hash           VARCHAR     NOT NULL,
    is_blocked          BOOL        NOT NULL DEFAULT FALSE,
    is_warned           BOOL        NOT NULL DEFAULT FALSE,
    is_deleted          BOOL        NOT NULL DEFAULT FALSE,
    is_in_order         BOOL        NOT NULL DEFAULT FALSE,
    current_rate        SMALLINT,
    picture             VARCHAR,
    cash                BIGINT      NOT NULL DEFAULT 0,
    term                BIGINT      NOT NULL DEFAULT 0,
    salary              BIGINT      NOT NULL DEFAULT 0,
    curator_id          BIGINT      REFERENCES curator(id),
    creation_datetime   TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE admin (
    id                  BIGSERIAL   PRIMARY KEY,
    name                VARCHAR     NOT NULL,
    surname             VARCHAR     NOT NULL,
    patronymic          VARCHAR     NOT NULL,
    phone               VARCHAR     NOT NULL UNIQUE,
    pass_hash           VARCHAR     NOT NULL,
    is_deleted          BOOL        NOT NULL DEFAULT FALSE,
    picture             VARCHAR,
    creation_datetime   TIMESTAMP   NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE restaurants (
    id                  BIGSERIAL   PRIMARY KEY,
    name                VARCHAR     NOT NULL,
    address             VARCHAR     NOT NULL,
    phone               VARCHAR     NOT NULL UNIQUE,
    pass_hash           VARCHAR     NOT NULL,
    location_lat        BIGSERIAL   NOT NULL,
    location_lng        BIGSERIAL   NOT NULL,
    working_from        TIME[]      NOT NULL,
    working_till        TIME[]      NOT NULL,
    is_working          BOOL        NOT NULL DEFAULT FALSE,
    is_deleted          BOOL        NOT NULL DEFAULT FALSE,
    creation_datetime   TIMESTAMP   NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TYPE PayMethod AS ENUM (
    'cash',
    'card',
    'already_payed'
);

CREATE TYPE TransportType AS ENUM (
    'car',
    'feet',
    'bicycle'
);

CREATE TYPE OrderStatus AS ENUM (
    'courier_finding',
    'courier_confirmation',
    'cooking',
    'ready_for_delivery',
    'delivering',
    'failure_by_courier',
    'failure_by_innocent',
    'success'
);

CREATE TABLE orders (
    id                  BIGSERIAL       PRIMARY KEY,
    restaurant_id       BIGSERIAL       REFERENCES restaurants(id) ON DELETE SET NULL,
    courier_id          BIGSERIAL       REFERENCES couriers(id) ON DELETE SET NULL,
    details             VARCHAR         NOT NULL,
    is_big_order        BOOL            NOT NULL DEFAULT FALSE,
    delivery_address    VARCHAR         NOT NULL,
    address_lat         BIGINT          NOT NULL,
    address_lng         BIGINT          NOT NULL,
    method              PayMethod      NOT NULL,
    courier_share       BIGINT          NOT NULL,
    order_price         BIGINT          NOT NULL,
    cooking_time        INTERVAL        NOT NULL,
    client_phone        VARCHAR         NOT NULL,
    client_comment      VARCHAR         NOT NULL,
    status              OrderStatus    NOT NULL DEFAULT 'courier_finding',
    finalize_comment    VARCHAR,
    take_datetime       TIMESTAMP,
    creation_datetime   TIMESTAMP       NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE orders_history (
    id                  BIGSERIAL       PRIMARY KEY,
    restaurant_id       BIGSERIAL       REFERENCES restaurants(id) ON DELETE SET NULL,
    courier_id          BIGSERIAL       REFERENCES couriers(id) ON DELETE SET NULL,
    details             VARCHAR         NOT NULL,
    is_big_order        BOOL            NOT NULL DEFAULT FALSE,
    delivery_address    VARCHAR         NOT NULL,
    address_lat         BIGINT          NOT NULL,
    address_lng         BIGINT          NOT NULL,
    method              PayMethod      NOT NULL,
    courier_share       BIGINT          NOT NULL,
    order_price         BIGINT          NOT NULL,
    cooking_time        TIME            NOT NULL,
    client_phone        VARCHAR         NOT NULL,
    client_comment      VARCHAR         NOT NULL,
    status              OrderStatus    NOT NULL,
    finalize_comment    VARCHAR,
    take_datetime       TIMESTAMP,
    creation_datetime   TIMESTAMP       NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE courier_rating (
    id                  BIGSERIAL       PRIMARY KEY,
    courier_id          BIGSERIAL       REFERENCES couriers(id) ON DELETE SET NULL,
    order_id            BIGSERIAL       REFERENCES orders_history(id) ON DELETE SET NULL,
    look                SMALLINT        NOT NULL,
    politeness          SMALLINT        NOT NULL
);

CREATE TABLE sessions (
    id                  BIGSERIAL       PRIMARY KEY,
    courier_id          BIGSERIAL       REFERENCES couriers(id) ON DELETE SET NULL,
    start_time          TIME            NOT NULL,
    end_time            TIME            NOT NULL,
    session_day         DATE            NOT NULL DEFAULT CURRENT_TIMESTAMP,
    transport           TransportType  NOT NULL DEFAULT 'feet'
);


CREATE TABLE sessions_history (
    id                  BIGSERIAL       PRIMARY KEY,
    courier_id          BIGSERIAL       REFERENCES couriers(id) ON DELETE SET NULL,
    start_datetime      TIMESTAMP       NOT NULL,
    end_datetime        TIMESTAMP       NOT NULL,
    session_day         DATE            NOT NULL DEFAULT CURRENT_TIMESTAMP,
    transport           TransportType  NOT NULL DEFAULT 'feet'
);

-- Второй батч

CREATE TABLE couriers_approvals (
    id          BIGSERIAL PRIMARY KEY,
    courier_id  BIGSERIAL REFERENCES couriers (id),
    order_id    BIGSERIAL REFERENCES orders (id),
    datetime    TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE OR REPLACE FUNCTION ban_couriers()
  RETURNS TRIGGER
  LANGUAGE PLPGSQL
  AS
$$BEGIN
	IF NEW.is_warned=OLD.is_warned THEN
		 IF OLD.is_warned=true THEN
             NEW.is_blocked=true;
         END IF;
	END IF;
	RETURN NEW;
END;$$;
--UPDATE couriers SET is_warned=true; -- test
CREATE TRIGGER trigger_ban_couriers
  BEFORE UPDATE
  ON couriers
  FOR EACH ROW
  EXECUTE PROCEDURE ban_couriers();

create or replace function process_approvals(
    arg_order_id BIGINT,
    arg_courier_id BIGINT
) returns void
language plpgsql
as $$ begin
    UPDATE couriers SET is_warned=true WHERE id IN (SELECT courier_id FROM couriers_approvals WHERE
        CURRENT_TIMESTAMP > couriers_approvals.datetime + (5 ||' minutes')::interval);
    DELETE FROM couriers_approvals WHERE courier_id IN (SELECT courier_id FROM couriers_approvals WHERE
        CURRENT_TIMESTAMP > couriers_approvals.datetime + (5 ||' minutes')::interval);
end;$$;

CREATE TEMP TABLE temp_for_suggest_orders (
    		    order_id BIGINT,
                restaurant_name VARCHAR,
                restaurant_address VARCHAR,
                restaurant_lat BIGINT,
                restaurant_lng BIGINT,
                destination_lat BIGINT,
                destination_lng BIGINT,
                destination_address VARCHAR,
                cooking_time TIME,
                payment_method PayMethod,
                pay_amount INT,
                distance BIGINT
    );
create or replace function find_suitable_orders(
   arg_lat bigint,
   arg_lng bigint,
   arg_courier_id bool,
   arg_count bigint
) returns temp_for_suggest_orders
language plpgsql
as $$
declare
    courier couriers := (SELECT * FROM couriers WHERE id=arg_courier_id);
    courier_session sessions := (SELECT * FROM sessions WHERE courier_id=arg_courier_id);
    ret temp_for_suggest_orders := (SELECT
                orders.id as order_id,
                r.name as restaurant_name,
                r.address as restaurant_address,
                r.location_lat as restaurant_lat,
                r.location_lng as restaurant_lng,
                orders.address_lat as destination_lat,
                orders.address_lng   as destination_lng,
                orders.delivery_address as destination_address,
                orders.cooking_time as cooking_time,
                orders.method as payment_method,
                orders.order_price as pay_amount,
                (2 * 3961 * ASIN(SQRT( POWER((SIN(RADIANS((r.location_lat - arg_lat) / 2))) , 2) + COS(RADIANS(arg_lat
        )) * COS(RADIANS(r.location_lat)) * POWER((SIN(RADIANS((r.location_lng - arg_lng) / 2))) , 2) ))) as distance
        FROM orders
            JOIN restaurants r on orders.restaurant_id = r.id
        WHERE status='courier_finding' and
              (CASE WHEN not courier_session.transport='car' THEN orders.is_big_order=true
		        ELSE TRUE
	        END)
        ORDER BY distance
        LIMIT arg_count);
begin
    IF courier.is_blocked THEN
        RAISE EXCEPTION 'user is blocked';
    END IF;
    IF courier.is_in_order THEN
        INSERT INTO couriers_approvals(courier_id,order_id) VALUES (arg_courier_id,ret.order_id);
    END IF;
    return ret;
end;$$;

create or replace function take_order(
    arg_order_id BIGINT,
    arg_courier_id BIGINT
) returns void
language plpgsql
as $$begin

    DELETE FROM couriers_approvals WHERE courier_id=arg_courier_id and arg_order_id=order_id;
    UPDATE
        orders
    SET
        courier_id=arg_courier_id,
        take_datetime=CURRENT_TIMESTAMP,
        status='cooking'
    WHERE
        id=arg_order_id;
    UPDATE couriers SET is_in_order=true WHERE id=arg_courier_id;
    return;
end;$$;

create or replace function pick_order(
    arg_order_id BIGINT,
    arg_courier_id BIGINT
) returns void
language plpgsql
as $$begin
    DELETE FROM couriers_approvals WHERE courier_id=arg_courier_id and arg_order_id=order_id;
    UPDATE
        orders
    SET
        courier_id=arg_courier_id,
        take_datetime=CURRENT_TIMESTAMP,
        status='cooking'
    WHERE
        id=arg_order_id;
    UPDATE couriers SET is_in_order=true WHERE id=arg_courier_id;
    return;
end;$$;

create or replace function set_ready_for_delivery_with_tick() returns void
language plpgsql
as $$ begin
    UPDATE
        orders
    SET
        status='ready_for_delivery'
    WHERE
        CURRENT_TIMESTAMP > orders.take_datetime + cooking_time;
    return;
end;$$;

create or replace function finalize_order(
    arg_order_id BIGINT,
    is_success BOOL,
    courier_fault BOOl,
    comment VARCHAR
) returns void
language plpgsql
as $$begin

    UPDATE
        orders
    SET
        orders.finalize_comment = comment,
        status=(
            CASE WHEN is_success THEN 'success' ELSE
                CASE WHEN courier_fault THEN 'failure_by_courier'
                    ELSE
                        'failure_by_innocent'
                    END
            END)
    WHERE
        id=arg_order_id;
    return;
end;$$;

create or replace function archieve_order(
    arg_order_id BIGINT
) returns bigint[]
language plpgsql
as $$
declare
    var_order orders := (SELECT * FROM orders WHERE orders.status= ANY(['success','failure_by_courier','failure_by_innocent']));
begin
    INSERT INTO orders_history VALUES (var_order);
    DELETE FROM orders WHERE id IN (SELECT id FROM var_order);
    return;
end;$$;

create or replace function archieve_session(
    arg_order_id BIGINT
) returns bigint[]
language plpgsql
as $$
declare
    var_order orders := (SELECT * FROM orders WHERE orders.status= ANY(['success','failure_by_courier','failure_by_innocent']));
begin
    INSERT INTO orders_history VALUES (var_order);
    DELETE FROM orders WHERE id IN (SELECT id FROM var_order);
    return;
end;$$;



CREATE OR REPLACE VIEW Pays AS
    SELECT
        ps.id as payment_id,
        ps.amount as total_summ,
        ps.order_status as status,
        ps.order_comment as comment,
        ps.order_type as ord_type,
        array_agg(JSONB_BUILD_OBJECT(
            'id', p2.id,
            'name', p2.name,
            'picture', p2.picture,
            'author', p2.author_name,
            'seller_id', p2.seller_id,
            'price', p2.price
        )) as paintings,
        ps.start_date as start_date,
        ps.end_date as end_date
    FROM
         paymentstatus as ps
            join paintingstopayments p
                on ps.id = p.pay_id
            join paintings p2
                on p.painting_id = p2.id
    GROUP BY
             ps.id,
             ps.amount,
             ps.order_status,
             ps.order_comment,
             ps.order_type,
             ps.start_date,
             ps.end_date;
