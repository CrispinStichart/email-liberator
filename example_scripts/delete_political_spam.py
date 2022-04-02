from email import message
import sys
import json
import re
from collections import namedtuple
from typing import cast
from enum import Enum
from dataclasses import dataclass

"""
This is an example script that will use regexes to determine if the email is a
political campaign donatation begging letter. If it is, it will request that
the email be deleted.
"""


class Email:
    address: list[str]
    subject: str
    body: str
    uid: int


@dataclass
class Message:
    Action = Enum("Action", "DELETE MOVE LABEL")
    # dataclass items
    uid: int
    actions: list[Action]
    stop: bool = False

    def __str__(self) -> str:
        return json.dumps(
            {"uid": self.uid, "actions": self.actions, "stop": self.stop},
            default=lambda x: x._name_,
        )


def text_in_last_percent(
    email: Email, text: str, last_percent: int, ignore_case=False
) -> bool:
    """
    Do a search for {text} in {email.body}, but only search in the last
    {last_percent} of the body.
    """
    body = email.body
    start_index = round(len(body) * (last_percent / 100))
    body = body[-start_index:]
    expr = rf"{text}"

    return bool(re.search(expr, body, flags=(re.IGNORECASE if ignore_case else 0)))


def test_for_political_spam(email) -> bool:
    # These patterns are a good indicator that the message is spam. Note that in
    # the second pattern I'm using python's string concatination feature to
    # split the regex across multiple lines.
    patterns = [
        r"paid for by actblue",
        (
            r"paid for by (((\w+\s*){1,4} ((\d\d(\d\d)?)|"
            r"(for \w+)))|(the ((democratic national (convention|committee))"
            r"|(dccc)))|((\w+\s*){1,6} PAC))"
        ),
    ]

    # Checking in the last 30 percent, because it's always in the footer.
    return any([text_in_last_percent(email, p, 30, ignore_case=True) for p in patterns])


def email_from_json(json_str: str) -> Email:
    json_dict = json.loads(json_str.strip())
    return cast(Email, namedtuple("Email", json_dict.keys())(*json_dict.values()))


def main():
    # Read from stdin forever
    line = sys.stdin.readline()
    email = email_from_json(line)
    if test_for_political_spam(email):
        print(Message(email.uid, [Message.Action.DELETE], True))


def test():
    test_in = r"""
        {"sender":["sender.bob@gmail.com"],"subject":"My first e-mail",
        "body":"blah TEST Blah blah aijld awidl awd bawbd iaw ilawd awd FOOBAR",
        "uid":69}
        """

    email = email_from_json(test_in)

    assert text_in_last_percent(email, "FOOBAR", 30)
    assert not text_in_last_percent(email, "TEST", 50)

    test_in2 = r"""
    {"sender":["sender.bob@gmail.com"],"subject":"My first e-mail",
    "body":"blah TEST Blah blah aijld awidl awd bawbd iaw ilawd awd FOOBAR
    blah TEST Blah blah aijld awidl awd bawbd iaw ilawd awd FOOBAR
    blah TEST Blah blah aijld awidl awd bawbd iaw ilawd awd FOOBAR
    blah TEST Blah blah aijld awidl awd bawbd iaw ilawd awd FOOBAR
    blah TEST Blah blah aijld awidl awd bawbd iaw ilawd awd FOOBAR
    blah TEST Blah blah aijld awidl awd bawbd iaw ilawd awd FOOBAR
    blah TEST Blah blah aijld awidl awd bawbd iaw ilawd awd FOOBAR
    blah TEST Blah blah aijld awidl awd bawbd iaw ilawd awd FOOBAR
    blah TEST Blah blah aijld awidl awd bawbd iaw ilawd awd FOOBAR
    paid for by the democratic national committee
    blah TEST Blah blah aijld awidl awd bawbd iaw ilawd awd FOOBAR",
    "uid":69}
    """.replace(
        "\n", ""
    )

    email2 = email_from_json(test_in2)

    assert not test_for_political_spam(email)
    assert test_for_political_spam(email2)

    _message = Message(69, [Message.Action.DELETE], True)


if __name__ == "__main__":
    main()
    # test()
