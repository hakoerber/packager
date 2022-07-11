#!/usr/bin/env python3

import time
import argparse
import code
import readline

from helium import *
import selenium

from selenium.webdriver.common.keys import Keys
from selenium.webdriver.common.by import By

parser = argparse.ArgumentParser()
parser.add_argument("--repl", action="store_true")
args = parser.parse_args()

opts = selenium.webdriver.firefox.options.Options()

for has_javascript in [True, False]:
    profile = selenium.webdriver.FirefoxProfile()
    profile.DEFAULT_PREFERENCES["frozen"]["javascript.enabled"] = has_javascript
    opts.profile = profile

    driver = selenium.webdriver.Remote(
        command_executor="http://localhost:4444/wd/hub", options=opts
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

        nameidx = next(
            i
            for i, v in enumerate(find_all(S("table > thead > tr > th")))
            if v.web_element.text == "Name"
        )
        descidx = next(
            i
            for i, v in enumerate(find_all(S("table > thead > tr > th")))
            if v.web_element.text == "Description"
        )

        write("newlist2", into=TextField(to_right_of="Name"))
        write("newlistdesc2", into=TextField(to_right_of="Description"))
        click(Button("Add"))

        lists_after = find_all(S("table > tbody > tr", below=Text("Package Lists")))
        assert len(lists_before) == len(lists_after) - 2

        new_entry1 = lists_after[-2]
        cells = new_entry1.web_element.find_elements_by_tag_name("td")
        assert cells[nameidx].text == "newlist"
        assert cells[descidx].text == "newlistdesc"

        new_entry2 = lists_after[-1]
        cells = new_entry2.web_element.find_elements_by_tag_name("td")
        assert cells[nameidx].text == "newlist2"
        assert cells[descidx].text == "newlistdesc2"

        editbtn = new_entry1.web_element.find_element_by_class_name("mdi-pencil")
        click(editbtn)

        lists_after = find_all(S("table > tbody > tr", below=Text("Package Lists")))
        editrow = find_all(S("table > tbody > tr"))[-2]

        inputs = editrow.web_element.find_elements_by_xpath("//input")
        write("editedname", into=inputs[0])
        write("editeddesc", into=inputs[1])

        savebtn = editrow.web_element.find_element_by_class_name("mdi-content-save")
        click(savebtn)

        lists_after = find_all(S("table > tbody > tr", below=Text("Package Lists")))
        new_entry1 = lists_after[-2]

        cells = new_entry1.web_element.find_elements_by_tag_name("td")
        assert cells[nameidx].text == "editedname"
        assert cells[descidx].text == "editeddesc"

        editbtn = new_entry1.web_element.find_element_by_class_name("mdi-pencil")
        click(editbtn)

        lists_after = find_all(S("table > tbody > tr", below=Text("Package Lists")))
        editrow = find_all(S("table > tbody > tr"))[-2]

        inputs = editrow.web_element.find_elements_by_xpath("//input")
        write("editedname_again", into=inputs[0])
        write("editeddesc_again", into=inputs[1])

        cancelbtn = editrow.web_element.find_element_by_class_name("mdi-cancel")
        click(cancelbtn)
        lists_after = find_all(S("table > tbody > tr", below=Text("Package Lists")))
        new_entry1 = lists_after[-2]

        cells = new_entry1.web_element.find_elements_by_tag_name("td")
        assert cells[nameidx].text == "editedname"
        assert cells[descidx].text == "editeddesc"

        lists_after = find_all(S("table > tbody > tr", below=Text("Package Lists")))

        new_entry1 = lists_after[-2]
        deletebtn = new_entry1.web_element.find_element_by_class_name("mdi-delete")
        click(deletebtn)

        lists_after = find_all(S("table > tbody > tr", below=Text("Package Lists")))

        assert len(lists_before) + 1 == len(lists_after)

        new_entry2 = lists_after[-1]
        deletebtn = new_entry2.web_element.find_element_by_class_name("mdi-delete")
        click(deletebtn)
        lists_after = find_all(S("table > tbody > tr", below=Text("Package Lists")))

        assert len(lists_before) == len(lists_after)

        if args.repl:
            code.InteractiveConsole(locals=globals()).interact()

    finally:
        driver.quit()


time.sleep(5)
