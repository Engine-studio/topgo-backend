DROP TABLE curators cascade;
DROP TABLE couriers cascade;
DROP TABLE admins cascade;
DROP TABLE restaurants cascade;
DROP TABLE orders cascade;
DROP TABLE couriers_approvals cascade;
DROP TABLE sessions cascade;
DROP TABLE courier_rating cascade;
DROP TABLE couriers_to_curators cascade;
DROP TYPE PayMethod cascade;
DROP TYPE TransportType cascade;
DROP TYPE OrderStatus cascade;


CREATE TABLE curators (
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
    current_rate_amount BIGINT      NOT NULL DEFAULT 0,
    current_rate_count  BIGINT      NOT NULL DEFAULT 0,
    picture             VARCHAR,
    cash                BIGINT      NOT NULL DEFAULT 0,
    term                BIGINT      NOT NULL DEFAULT 0,
    salary              BIGINT      NOT NULL DEFAULT 0,
    creation_datetime   TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE couriers_to_curators (
    id                  BIGSERIAL   PRIMARY KEY,
    courier_id          BIGINT      REFERENCES couriers(id),
    curator_id          BIGINT      REFERENCES curators(id)
);

CREATE TABLE admins (
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
    location_lat        DOUBLE PRECISION   NOT NULL,
    location_lng        DOUBLE PRECISION   NOT NULL,
    working_from        TIME[]      NOT NULL,
    working_till        TIME[]      NOT NULL,
    is_working          BOOL        NOT NULL DEFAULT FALSE,
    is_deleted          BOOL        NOT NULL DEFAULT FALSE,
    creation_datetime   TIMESTAMP   NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TYPE PayMethod AS ENUM (
    'Cash',
    'Card',
    'AlreadyPayed'
);

CREATE TYPE TransportType AS ENUM (
    'Car',
    'Feet',
    'Bicycle'
);


CREATE TYPE OrderStatus AS ENUM (
    'CourierFinding',
    'CourierConfirmation',
    'Cooking',
    'ReadyForDelivery',
    'Delivering',
    'Delivered',
    'FailureByCourier',
    'FailureByRestaurant',
    'Success'
);

CREATE TABLE sessions (
    id                  BIGSERIAL       PRIMARY KEY,
    courier_id          BIGSERIAL       REFERENCES couriers(id) ON DELETE SET NULL,
    start_time          TIME            NOT NULL,
    end_time            TIME            NOT NULL,
    session_day         DATE            NOT NULL DEFAULT CURRENT_TIMESTAMP,
    transport           TransportType  NOT NULL DEFAULT 'Feet'
);

CREATE TABLE orders (
    id                  BIGSERIAL       PRIMARY KEY,
    restaurant_id       BIGSERIAL       REFERENCES restaurants(id) ON DELETE SET NULL,
    session_id          BIGSERIAL       REFERENCES sessions(id) ON DELETE SET NULL,
    details             VARCHAR         NOT NULL,
    is_big_order        BOOL            NOT NULL DEFAULT FALSE,
    delivery_address    VARCHAR         NOT NULL,
    address_lat         DOUBLE PRECISION          NOT NULL,
    address_lng         DOUBLE PRECISION          NOT NULL,
    method              PayMethod      NOT NULL,
    courier_share       BIGINT          NOT NULL,
    order_price         BIGINT          NOT NULL,
    cooking_time        TIME            NOT NULL,
    client_phone        VARCHAR         NOT NULL,
    client_comment      VARCHAR         NOT NULL,
    status              OrderStatus    NOT NULL DEFAULT 'CourierFinding',
    finalize_comment    VARCHAR,
    finalize_datetime   TIMESTAMP,
    take_datetime       TIMESTAMP,
    creation_datetime   TIMESTAMP       NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE courier_rating (
    id                  BIGSERIAL       PRIMARY KEY,
    courier_id          BIGSERIAL       REFERENCES couriers(id) ON DELETE SET NULL,
    order_id            BIGSERIAL       REFERENCES orders(id) ON DELETE SET NULL UNIQUE,
    look                SMALLINT        NOT NULL,
    politeness          SMALLINT        NOT NULL
);

CREATE TABLE couriers_approvals (
    id          BIGSERIAL PRIMARY KEY,
    courier_id  BIGSERIAL REFERENCES couriers (id),
    order_id    BIGSERIAL REFERENCES orders (id),
    datetime    TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE notifications (
    id                  BIGSERIAL       PRIMARY KEY,
    message             VARCHAR         NOT NULL,
    creation_datetime   TIMESTAMP       NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE notifications_to_couriers (
    id                  BIGSERIAL       PRIMARY KEY,
    courier_id          BIGSERIAL       REFERENCES couriers(id) ON DELETE SET NULL,
    notific_id          BIGSERIAL       REFERENCES notifications(id) ON DELETE SET NULL
);

SELECT * FROM get_notification(4);
create or replace function get_notification(
    arg_courier_id BIGINT
) returns TABLE (message VARCHAR)
language plpgsql
as $$
declare
    courier timestamp;
    notif bigint[];
begin
    SELECT couriers.creation_datetime FROM couriers WHERE id=arg_courier_id INTO courier;
    SELECT array_agg(n.id) FROM notifications n
    WHERE n.creation_datetime > courier AND n.id NOT IN (
        SELECT notifications_to_couriers.notific_id FROM notifications_to_couriers WHERE courier_id=arg_courier_id
        )
    INTO notif;
    INSERT INTO notifications_to_couriers (courier_id,notific_id)
        SELECT arg_courier_id, notifications.id FROM notifications WHERE notifications.id=ANY(notif);
    return QUERY SELECT notifications.message FROM notifications WHERE notifications.id=ANY(notif);
end;$$;

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

CREATE OR REPLACE FUNCTION roll_to_finding()
  RETURNS TRIGGER
  LANGUAGE PLPGSQL
  AS
$$BEGIN
	UPDATE orders SET status='CourierFinding' WHERE id=OLD.order_id AND status='CourierConfirmation';
	RETURN OLD;
END;$$;

CREATE TRIGGER trigger_roll_to_finding_d
  BEFORE DELETE
  ON couriers_approvals
  FOR EACH ROW
  EXECUTE PROCEDURE roll_to_finding();

CREATE OR REPLACE FUNCTION proc_rating()
  RETURNS TRIGGER
  LANGUAGE PLPGSQL
  AS
$$BEGIN
	UPDATE couriers SET current_rate_amount =
	    (current_rate_amount + NEW.look + NEW.politeness),
	    current_rate_count = (current_rate_count + 2) WHERE id = NEW.courier_id;
	RETURN NEW;
END;$$;


CREATE TRIGGER trigger_proc_rating
  AFTER INSERT
  ON courier_rating
  FOR EACH ROW
  EXECUTE PROCEDURE proc_rating();

create or replace function process_approvals() returns void
language plpgsql
as $$ begin
    UPDATE couriers SET is_warned=true WHERE id IN (SELECT courier_id FROM couriers_approvals
        JOIN couriers ON couriers_approvals.courier_id = couriers.id WHERE
        CURRENT_TIMESTAMP > couriers_approvals.datetime + (5 ||' minutes')::interval AND not couriers.is_in_order);
    DELETE FROM couriers_approvals WHERE courier_id IN (SELECT courier_id FROM couriers_approvals WHERE
        CURRENT_TIMESTAMP > couriers_approvals.datetime + (5 ||' minutes')::interval);
end;$$;

select * FROM find_suitable_orders(0,0,3);
create or replace function find_suitable_orders(
   arg_lat DOUBLE PRECISION,
   arg_lng DOUBLE PRECISION,
   arg_courier_id bigint
) returns TABLE (
    		    order_id BIGINT,
                restaurant_name VARCHAR,
                restaurant_address VARCHAR,
                restaurant_lat DOUBLE PRECISION,
                restaurant_lng DOUBLE PRECISION,
                destination_lat DOUBLE PRECISION,
                destination_lng DOUBLE PRECISION,
                destination_address VARCHAR,
                cooking_time TIME,
                payment_method PayMethod,
                pay_amount BIGINT,
                distance DOUBLE PRECISION
) language plpgsql as $$
declare
    courier couriers;
    courier_session sessions;
    ret bigint;
begin
    SELECT * FROM couriers WHERE id=arg_courier_id INTO courier;
    SELECT * FROM sessions WHERE courier_id=arg_courier_id
                                                          AND sessions.session_day = CURRENT_DATE
                                                          AND sessions.end_time > CURRENT_TIME INTO courier_session;
    SELECT
                orders.id
        FROM orders
            JOIN restaurants r on orders.restaurant_id = r.id
        WHERE status='CourierFinding' and
              (CASE WHEN not courier_session.transport='Car' THEN orders.is_big_order=false
		        ELSE TRUE
	        END)
        GROUP BY orders.id,r.location_lat,r.location_lng
        ORDER BY (2 * 3961 * ASIN(SQRT( POWER((SIN(RADIANS((r.location_lat - arg_lat) / 2))) , 2) + COS(RADIANS(arg_lat
        )) * COS(RADIANS(r.location_lat)) * POWER((SIN(RADIANS((r.location_lng - arg_lng) / 2))) , 2) )))
        INTO ret;
    IF courier.is_blocked THEN
        RAISE EXCEPTION 'user is blocked';
    END IF;
    INSERT INTO couriers_approvals(courier_id,order_id) VALUES (arg_courier_id,ret);
    UPDATE
        orders
    SET
        session_id=courier_session.id,
        status='CourierConfirmation'
    WHERE
        id=ret;
    return QUERY SELECT
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
        WHERE orders.id=ret;
end; $$;
SELECT * FROM take_order(1,3);
create or replace function take_order(
    arg_order_id BIGINT,
    arg_courier_id BIGINT
) returns void
language plpgsql
as $$begin
    UPDATE couriers SET is_in_order=true WHERE id=arg_courier_id;
    UPDATE
        orders
    SET
        take_datetime=CURRENT_TIMESTAMP,
        status='Cooking'
    WHERE
        id=arg_order_id;
    DELETE FROM couriers_approvals WHERE courier_id=arg_courier_id and arg_order_id=order_id;
    return;
end;$$;

create or replace function refuse_order(
    arg_order_id BIGINT,
    arg_courier_id BIGINT
) returns void
language plpgsql
as $$
declare
    courier couriers;
begin
    SELECT * FROM couriers WHERE id=arg_courier_id INTO courier;
    DELETE FROM couriers_approvals WHERE courier_id=arg_courier_id and arg_order_id=order_id;
    IF not courier.is_in_order THEN
        UPDATE couriers SET is_warned=true WHERE id=arg_courier_id;
    END IF;
    return;
end;$$;

create or replace function set_ready_for_delivery(
    arg_order_id bigint
) returns void
language plpgsql
as $$ begin
    UPDATE
        orders
    SET
        status='ReadyForDelivery'
    WHERE
        id=arg_order_id AND status='Cooking';
    return;
end;$$;

create or replace function pick_order(
    arg_order_id BIGINT,
    arg_courier_id BIGINT
) returns void
language plpgsql
as $$begin
    UPDATE
        orders
    SET
        orders.courier_id=arg_courier_id,
        take_datetime=CURRENT_TIMESTAMP,
        status='Delivering'
    WHERE
        id=arg_order_id AND status='ReadyForDelivery';
    return;
end;$$;

create or replace function set_delivered(
    arg_order_id BIGINT,
    arg_courier_id BIGINT
) returns void
language plpgsql
as $$begin
    UPDATE
        orders
    SET
        status='Delivered'
    WHERE
        id=arg_order_id AND status='Delivering';
    return;
end;$$;

create or replace function finalize_order(
    arg_order_id BIGINT,
    is_success BOOL,
    courier_fault BOOl,
    comment VARCHAR
) returns void
language plpgsql
as $$
declare
    var_order orders;
    var_courier couriers;
begin

    UPDATE
        orders
    SET
        orders.finalize_comment = comment,
        orders.finalize_datetime = CURRENT_TIMESTAMP,
        status=(
            CASE WHEN is_success THEN 'Success' ELSE
                CASE WHEN courier_fault THEN 'FailureByCourier'
                    ELSE
                        'FailureByRestaurant'
                    END
            END)
    WHERE
        id=arg_order_id AND (status='Delivered' OR not is_success);
    SELECT * FROM orders WHERE id=arg_order_id INTO var_order;

    var_courier = (SELECT courier_id FROM orders o JOIN sessions s
                ON o.session_id=s.courier_id JOIN couriers c ON c.id=s.courier_id
                WHERE o.id=var_order.id);

    IF var_order.status = 'Success' THEN
        UPDATE couriers SET couriers.salary= couriers.salary + (var_order.courier_share - 1500) WHERE couriers.id=var_courier.id;
        IF var_order.method='Cash' THEN
            UPDATE couriers SET couriers.cash = couriers.cash + var_order.order_price WHERE couriers.id=var_courier.id;
        END IF;
        IF var_order.method='Card' THEN
            UPDATE couriers SET couriers.term = couriers.term + var_order.order_price WHERE couriers.id=var_courier.id;
        END IF;
    END IF;
    IF var_order.status = 'FailureByRestaurant' THEN
        UPDATE couriers SET couriers.salary= couriers.salary + (var_order.courier_share - 1500) WHERE couriers.id=var_courier.id;
    END IF;
    UPDATE couriers SET is_in_order=false WHERE id IN (
        SELECT courier_id
        FROM orders JOIN sessions s on orders.session_id = s.id
        WHERE s.courier_id=var_courier.id);
    return;
end;$$;
