module Main exposing (..)

-- import Dict exposing (..)

import Msg exposing (..)
import Model exposing (..)
import Update exposing (..)
import Subscription exposing (..)
import View exposing (..)
import Html exposing (program)


-- App


main : Program Never Model Msg
main =
    Html.program
        { init = init
        , view = view
        , update = update
        , subscriptions = subscriptions
        }



-- Update
-- Subscriptions
-- View
