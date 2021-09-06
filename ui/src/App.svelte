<svelte:head>
    <link href="https://unpkg.com/tailwindcss@^2/dist/tailwind.min.css" rel="stylesheet">
</svelte:head>

<script lang="ts">
    import PackageLists from "./routes/PackageLists.svelte";
    import Home from "./routes/Home.svelte";

    $: {
        const path = window.location.pathname;
        console.log(`Routing ${path}`);

        let component;

        if (path == "/") {
            compontent = Home;
        } else if (path == "/lists/") {
            console.log("Routing to package lists");
            component = PackageLists;
        } else if (path.startswith("/lists/")) {
            console.log("hm");
        } else {
            console.log("No matching route found");
        }
    }

</script>

<main>
    <svelte:component this={component} />
</main>
