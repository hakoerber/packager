module Main exposing (..)

-- import Dict exposing (..)

import Html exposing (..)
import Html.Events exposing (..)
import Html.Attributes exposing (..)
import Http
import Json.Decode
import Json.Decode.Pipeline
import Json.Encode


-- Constants


restEndpoint : String
restEndpoint =
    "http://localhost:8000/api/v1/lists/"



-- restPaths : { a : String }
-- restPaths =
--     { lists = "lists" }


httpHeaders : List Http.Header
httpHeaders =
    [ Http.header "Access-Control-Allow-Origin" "*"
    ]



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
    , err : Maybe Http.Error
    , text : String
    , deleteList : Maybe PkgList
    }


init : ( Model, Cmd Msg )
init =
    ( Model [] Nothing Nothing "" Nothing, getLists )


type alias PkgList =
    { id : Int
    , name : String
    }


type alias NewPkgList =
    { name : String
    }


decodePkgList : Json.Decode.Decoder PkgList
decodePkgList =
    Json.Decode.Pipeline.decode PkgList
        |> Json.Decode.Pipeline.required "id" Json.Decode.int
        |> Json.Decode.Pipeline.required "name" Json.Decode.string


encodePkgList : PkgList -> Json.Encode.Value
encodePkgList record =
    Json.Encode.object
        [ ( "name", Json.Encode.string <| record.name )
        , ( "id", Json.Encode.int <| record.id )
        ]


encodeNewPkgList : NewPkgList -> Json.Encode.Value
encodeNewPkgList newPkgList =
    Json.Encode.object
        [ ( "name", Json.Encode.string <| newPkgList.name )
        ]



-- Update


type Msg
    = GetLists
    | OnGetLists (Result Http.Error (List PkgList))
    | AddList String
    | OnAddList (Result Http.Error PkgList)
    | TextInput String
    | DeleteList PkgList
    | OnDeleteList (Result Http.Error ())


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case msg of
        GetLists ->
            ( model, getLists )

        OnGetLists (Ok newlists) ->
            ( { model | lists = newlists }, Cmd.none )

        OnGetLists (Err result) ->
            ( model, Cmd.none )

        AddList name ->
            ( model, addList (NewPkgList name) )

        OnAddList (Ok list) ->
            let
                newmodel1 =
                    { model | lists = model.lists ++ [ list ] }

                newmodel2 =
                    { newmodel1 | text = "" }
            in
                ( newmodel2, Cmd.none )

        OnAddList (Err result) ->
            ( { model | err = Just result }, Cmd.none )

        TextInput text ->
            ( { model | text = text }, Cmd.none )

        DeleteList list ->
            ( { model | deleteList = Just list }, removeList list )

        OnDeleteList (Ok ()) ->
            let
                newModel1 =
                    { model | lists = List.filter (\f -> (Just f) /= model.deleteList) model.lists }

                newModel2 =
                    { newModel1 | deleteList = Nothing }
            in
                ( newModel2, Cmd.none )

        -- ( model, Cmd.none )
        OnDeleteList (Err result) ->
            ( { model | err = Just result }, Cmd.none )



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
        , ( "margin", "20px" )
        ]


buttonStyle : Attribute Msg
buttonStyle =
    style
        [ ( "margin", "20px" ) ]


inputStyle : Attribute Msg
inputStyle =
    style
        [ ( "margin", "20px" ) ]


headerStyle : Attribute Msg
headerStyle =
    style
        [ ( "margin", "20px" ) ]


view : Model -> Html Msg
view model =
    div []
        [ h1 [ headerStyle ] [ text "Lists" ]
        , button [ onClick GetLists, buttonStyle ] [ text "Reload Lists" ]
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
                                    , td [ tableStyle ] [ button [ onClick (DeleteList l) ] [ text "X" ] ]
                                    ]
                            )
                   )
        , input [ placeholder "New List Name", onInput TextInput, value model.text, inputStyle ] []
        , button [ onClick (AddList model.text), buttonStyle ] [ text "Add New List" ]
        ]



-- HTTP


getLists : Cmd Msg
getLists =
    let
        request =
            Http.request
                { method = "GET"
                , headers = httpHeaders
                , url = restEndpoint
                , body = Http.emptyBody
                , expect = (Http.expectJson (Json.Decode.list decodePkgList))
                , timeout = Nothing
                , withCredentials = False
                }
    in
        Http.send OnGetLists request


addList : NewPkgList -> Cmd Msg
addList pkgList =
    let
        headers =
            [ Http.header "Access-Control-Allow-Origin" "*"
            ]

        request =
            Http.request
                { method = "POST"
                , headers = httpHeaders
                , url = restEndpoint
                , body = (Http.jsonBody (encodeNewPkgList pkgList))
                , expect = (Http.expectJson decodePkgList)
                , timeout = Nothing
                , withCredentials = False
                }
    in
        Http.send OnAddList request


removeList : PkgList -> Cmd Msg
removeList pkgList =
    let
        headers =
            [ Http.header "Access-Control-Allow-Origin" "*"
            ]

        request =
            Http.request
                { method = "DELETE"
                , headers = httpHeaders
                , url = restEndpoint
                , body = (Http.jsonBody (encodePkgList pkgList))
                , expect = Http.expectStringResponse (\_ -> Ok ())
                , timeout = Nothing
                , withCredentials = False
                }
    in
        Http.send OnDeleteList request
