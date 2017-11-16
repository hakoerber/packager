module Main exposing (..)

-- import Dict exposing (..)

import Html exposing (..)
import Html.Events exposing (..)
import Html.Attributes exposing (..)
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
    { lists : List PkgList
    , newList : Maybe NewPkgList
    , err : Maybe String
    }


init : ( Model, Cmd Msg )
init =
    ( Model [] Nothing Nothing, Cmd.none )


type alias PkgList =
    { id : Int
    , name : String
    }


type alias NewPkgList =
    { name : String
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
    | AddList


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case msg of
        LoadLists ->
            ( model, getLists )

        GetLists (Ok newlists) ->
            ( { model | lists = newlists }, Cmd.none )

        GetLists (Err result) ->
            ( model, Cmd.none )

        AddList ->
            ( { model | newList = Just (NewPkgList "hi") }, Cmd.none )



-- Subscriptions


subscriptions : Model -> Sub Msg
subscriptions model =
    Sub.none



-- View


tableStyle : Attribute Msg
tableStyle =
    style
        [ ( "border", "1px solid black" )
        , ( "padding", "5px" )
        ]


view : Model -> Html Msg
view model =
    div []
        [ h1 [] [ text "Lists" ]
        , button [ onClick LoadLists ] [ text "Reload Lists" ]
        , table [ tableStyle ] <|
            [ tr
                [ tableStyle ]
                [ th [ tableStyle ] [ text "ID" ]
                , th [ tableStyle ] [ text "Name" ]
                ]
            ]
                ++ (model.lists
                        |> List.map
                            (\l ->
                                tr
                                    [ tableStyle ]
                                    [ td [ tableStyle ] [ l.id |> toString |> text ]
                                    , td [ tableStyle ] [ text l.name ]
                                    ]
                            )
                   )
        , input [ placeholder "New List Name" ] []
        , button [ onClick AddList ] [ text "Add New List" ]
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
