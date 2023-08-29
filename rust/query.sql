INSERT INTO
    trips_to_trips_types (trip_id, trip_type_id)
SELECT trips.id as trip_id, trips_types.id as trip_type_id
    FROM trips
    INNER JOIN trips_types
    WHERE 
        trips.id = "629ac022-d715-4dfa-9a53-56ffd0be36f3" 
        AND trips.user_id = "f981b589-4ed4-42e7-85f0-453961849fc4"
        AND trips_types.id = "bdcc58ee-0f09-4fea-ab76-50f3e118228e"
        AND trips_types.user_id = "f981b589-4ed4-42e7-85f0-453961849fc4"
