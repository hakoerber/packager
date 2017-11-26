module Msg exposing (..)

import Http
import Model.PkgList exposing (..)


type Msg
    = GetLists
    | OnGetLists (Result Http.Error (List PkgList))
    | AddList String
    | OnAddList (Result Http.Error PkgList)
    | TextInput String
    | DeleteList PkgList
    | OnDeleteList (Result Http.Error ())
