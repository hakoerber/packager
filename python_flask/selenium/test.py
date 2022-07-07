#!/usr/bin/env python3

import time

from helium import *
import selenium

from selenium.webdriver.common.keys import Keys
from selenium.webdriver.common.by import By

opts = selenium.webdriver.firefox.options.Options()

profile = selenium.webdriver.FirefoxProfile()
profile.set_preference("javascript.enabled", "false")
profile.DEFAULT_PREFERENCES['frozen']['javascript.enabled'] = False
opts.profile = profile

driver = selenium.webdriver.Remote(
    command_executor="http://localhost:4444/wd/hub",
    options=opts,
)
driver.implicitly_wait(0)
Config.implicit_wait_secs = 1

try:
    helium.set_driver(driver)
    helium.go_to("http://localhost:5000")

    assert driver.title == "Packager"

    new_entry = Text("Add new package list")

    lists_before = find_all(S("table > tbody > tr", below=Text("Package Lists")))

    write("newlist", into=TextField(to_right_of="Name"))
    write("newlistdesc", into=TextField(to_right_of="Description"))
    click(Button("Add"))

    lists_after = find_all(S("table > tbody > tr", below=Text("Package Lists")))

    assert len(lists_before) == len(lists_after) - 1

    nameidx = next(i for i,v in enumerate(find_all(S("table > thead > tr > th"))) if v.web_element.text == "Name")
    descidx = next(i for i,v in enumerate(find_all(S("table > thead > tr > th"))) if v.web_element.text == "Description")

    new_entry = lists_after[-1]

    cells = new_entry.web_element.find_elements_by_tag_name("td")

    assert cells[nameidx].text == "newlist"
    assert cells[descidx].text == "newlistdesc"

    lists_before = lists_after

    deletebtn = new_entry.web_element.find_element_by_class_name("mdi-delete")

    click(deletebtn)

    lists_after = find_all(S("table > tbody > tr", below=Text("Package Lists")))

    assert len(lists_before) - 1 == len(lists_after)

    import code; code.interact(local=locals())
    time.sleep(5)
finally:
    driver.quit()
