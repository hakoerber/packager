module Backend.Rest exposing (..)

import Model.PkgList exposing (..)
import Json.Decode
import Http
import Msg exposing (..)


-- HTTP


restEndpoint : String
restEndpoint =
    "http://localhost:8000/api/v1/lists/"


httpHeaders : List Http.Header
httpHeaders =
    [ Http.header "Access-Control-Allow-Origin" "*"
    ]


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
        request =
            Http.request
                { method = "DELETE"
                , headers = httpHeaders
                , url = (String.concat [ restEndpoint, (toString pkgList.id), "/" ])
                , body = (Http.jsonBody (encodePkgList pkgList))
                , expect = Http.expectStringResponse (\_ -> Ok ())
                , timeout = Nothing
                , withCredentials = False
                }
    in
        Http.send OnDeleteList request
