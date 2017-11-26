module View exposing (..)

import Msg exposing (..)
import Model exposing (..)
import Html exposing (..)
import Html.Events
    exposing
        ( onInput
        , onClick
        )
import Html.Attributes
    exposing
        ( value
        , placeholder
        , style
        )


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
