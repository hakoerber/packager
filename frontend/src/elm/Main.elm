module Main exposing (..)

import Html exposing (..)
import Html.Events exposing (..)
import Random exposing (..)


-- App


main : Program Never Model Msg
main =
    Html.program
        { init = init
        , view = view
        , update = update
        , subscriptions = subscriptions
        }



-- Model


type alias Model =
    { lists : List Int }


init : ( Model, Cmd Msg )
init =
    ( Model [], Cmd.none )



-- Update


type Msg
    = LoadLists
    | GetLists (List Int)


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case msg of
        LoadLists ->
            ( model
            , Random.generate GetLists (Random.list 6 (int 0 100))
            )

        GetLists lists ->
            ( model
            , Cmd.none
            )



-- Subscriptions


subscriptions : Model -> Sub Msg
subscriptions model =
    Sub.none



-- View


view : Model -> Html Msg
view model =
    div []
        [ h1 [] [ text "lists" ]
        , button [ onClick LoadLists ] [ text "load lists" ]
        , div [] [ text (toString model.lists) ]
        ]
