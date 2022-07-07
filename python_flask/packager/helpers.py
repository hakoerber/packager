def cls(*args):
    return " ".join([a for a in args if a is not None])


def jsbool(b):
    return "true" if b else "false"


def alpinedata(d):
    elements = []
    for k, v in d.items():
        elements.append(f"{k}: " + v)

    return "{" + ",".join(elements) + "}"
