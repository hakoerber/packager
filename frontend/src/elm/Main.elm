module Main exposing (..)

-- import Dict exposing (..)

import Html exposing (..)
import Html.Events exposing (..)
import Http
import Json.Decode as JDec
import Json.Decode.Pipeline as JDecP


-- import JDec
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
    { lists : List PkgList }


init : ( Model, Cmd Msg )
init =
    ( Model [], Cmd.none )


type alias PkgList =
    { id : Int
    , name : String
    }


decodePkgList : JDec.Decoder PkgList
decodePkgList =
    JDecP.decode PkgList
        |> JDecP.required "id" JDec.int
        |> JDecP.required "name" JDec.string



-- Update


type Msg
    = LoadLists
    | GetLists (Result Http.Error (List PkgList))


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case msg of
        LoadLists ->
            ( model, getLists )

        GetLists (Ok newlists) ->
            ( Model newlists, Cmd.none )

        GetLists (Err result) ->
            ( model, Cmd.none )



-- Subscriptions


subscriptions : Model -> Sub Msg
subscriptions model =
    Sub.none



-- View


view : Model -> Html Msg
view model =
    div []
        [ h1 [] [ text "lists" ]
        , button [ onClick LoadLists ] [ text "reload lists" ]
        , ul []
            (model.lists
                |> List.map
                    (\l ->
                        li []
                            [ b []
                                [ ((l.id |> toString) ++ ": ") |> text
                                ]
                            , text l.name
                            ]
                    )
            )
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
                , expect = (Http.expectJson (JDec.list decodePkgList))
                , timeout = Nothing
                , withCredentials = False
                }
    in
        Http.send GetLists request
