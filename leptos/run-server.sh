DATABASE_URL="postgresql://packager@postgres/packager?host=$PWD/../pgdata/run" cargo --color=always leptos watch --server-only "${@}"
