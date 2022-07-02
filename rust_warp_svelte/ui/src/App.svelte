<svelte:head>
    <link href="https://unpkg.com/tailwindcss@^2/dist/tailwind.min.css" rel="stylesheet">
</svelte:head>

<script lang="ts">
    import Home from "./routes/Home.svelte";
    import PackageLists from "./routes/PackageLists.svelte";
    import PackageList from "./routes/PackageList.svelte";
    import Preparation from "./routes/Preparation.svelte";
    import Trips from "./routes/Trips.svelte";
    import Trip from "./routes/Trip.svelte";
    import NotFound from "./routes/NotFound.svelte";

    function normalize(path) {
        return path.replace(/\/+$/, '') + "/";
    }

    let currentRoute;
    let data;

    function route(path) {
        path = normalize(path);
        console.log(`Routing path "${path}"`);
        data = {}

        let urlParts = path.split("/").slice(1, -1);

        if (path === "/") {
            console.log("=> Home");
            currentRoute = Home;
        } else if (urlParts[0] == "lists" && urlParts.length == 1) {
            console.log("=> PackageLists");
            currentRoute = PackageLists;
        } else if (urlParts[0] == "trips" && urlParts.length == 1) {
            console.log("=> Trips");
            currentRoute = Trips;
        } else if (urlParts[0] == "trips" && urlParts.length == 2) {
            console.log("=> Trip");
            currentRoute = Trip;
            data = {id: urlParts[1]};
        } else if (urlParts[0] == "lists" && urlParts.length == 2) {
            console.log("=> PackageList");
            currentRoute = PackageList;
            data = {id: urlParts[1]};
        } else if (urlParts.length == 5
                && urlParts[0] == "lists"
                && urlParts[2] == "items"
                && urlParts[4] == "preparation") {
            console.log("=> PackageList");
            currentRoute = Preparation;
            data = {list_id: urlParts[1], item_id: urlParts[3]};
        } else {
            console.log("No matching route found");
            currentRoute = NotFound;
        }
    }

    window.onload = e => {
        route(window.location.pathname);
    }

    function redirect(path) {
        history.pushState({id: path}, "", path);
        route(path);
    }

    window.addEventListener("locationchange", function() {
        route(window.location.pathname);
    });

    window.addEventListener("popstate", event => {
        route(window.location.pathname);
    });

</script>

<main>
    <svelte:component this={currentRoute} redirect={redirect} data={data}/>
</main>
