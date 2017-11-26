module Model.PkgList exposing (..)

import Json.Decode
import Json.Decode.Pipeline
import Json.Encode


type alias PkgList =
    { id : Int
    , name : String
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


type alias NewPkgList =
    { name : String
    }


encodeNewPkgList : NewPkgList -> Json.Encode.Value
encodeNewPkgList newPkgList =
    Json.Encode.object
        [ ( "name", Json.Encode.string <| newPkgList.name )
        ]
