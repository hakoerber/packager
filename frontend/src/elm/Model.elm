module Model exposing (..)

import Http
import Msg exposing (..)
import Model.PkgList exposing (..)
import Backend.Rest exposing (..)


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
