module Update exposing (..)

import Model.PkgList exposing (..)
import Backend.Rest exposing (..)
import Model exposing (..)
import Msg exposing (..)


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case msg of
        GetLists ->
            ( model, getLists )

        OnGetLists (Ok newlists) ->
            ( { model | lists = newlists }, Cmd.none )

        OnGetLists (Err result) ->
            ( { model | err = Just result }, Cmd.none )

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
