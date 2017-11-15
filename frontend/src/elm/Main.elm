module Main exposing (..)

import Html exposing (..)
import Html.Events exposing (..)
import Http
import Json.Decode


-- import Json.Decode
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
    { lists : List String }


init : ( Model, Cmd Msg )
init =
    ( Model [], Cmd.none )



-- Update


type Msg
    = LoadLists
    | GetLists (Result Http.Error (List String))


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case msg of
        LoadLists ->
            ( model, getLists )

        GetLists (Ok newlists) ->
            ( Model newlists, Cmd.none )

        GetLists (Err result) ->
            ( Model [ toString result ], Cmd.none )



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
        , ul []
            (List.map (\l -> li [] [ text l ]) model.lists)
        ]



-- HTTP


getLists : Cmd Msg
getLists =
    let
        headers =
            [ Http.header "Access-Control-Allow-Origin" "*"
            ]

        url =
            "http://localhost:8000/api/v1/lists/"

        request =
            Http.request
                { method = "GET"
                , headers = headers
                , url = url
                , body = Http.emptyBody
                , expect = Http.expectJson (Json.Decode.list Json.Decode.string)
                , timeout = Nothing
                , withCredentials = False
                }
    in
        Http.send GetLists request
