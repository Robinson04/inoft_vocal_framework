from json import dumps as json_dumps

from inoft_vocal_framework.platforms_handlers.current_platform_static_data import CurrentPlatformData, RefsAvailablePlatforms
from inoft_vocal_framework.platforms_handlers.endpoints_providers.providers import LambdaResponseWrapper


class Card:
    def __init__(self):
        self._type = None
        self._title = str()
        self._content = str()

    @property
    def type(self):
        return self._type

    @type.setter
    def type(self, type_value) -> None:
        self._type = type_value

    @property
    def title(self):
        return self._title

    @title.setter
    def title(self, title_string: str) -> None:
        self._title = title_string

    @property
    def content(self):
        return self._content

    @content.setter
    def content(self, content_string: str) -> None:
        self._content = content_string

class OutputSpeech:
    TYPE_KEY_TEXT = "PlainText"
    TYPE_KEY_SSML = "SSML"

    def __init__(self):
        self._type = None
        self._text = str()
        self._ssml = str()

    @property
    def type(self):
        return self._type

    @property
    def text(self):
        return self._text

    @property
    def ssml(self):
        return self._ssml

    def set_text(self, text: str):
        if not isinstance(text, str):
            raise Exception(f"The following text is not a str object : {text}")
        self._type = self.TYPE_KEY_TEXT
        self._text = text
        return self

    def set_ssml(self, ssml_string: str):
        if not isinstance(ssml_string, str):
            raise Exception(f"The following ssml_string is not a str object : {ssml_string}")
        self._type = self.TYPE_KEY_SSML
        self._ssml = ssml_string
        return self

    def set_based_on_type(self, value_to_set: str, type_key: str):
        if type_key == self.TYPE_KEY_SSML:
            self.set_ssml(ssml_string=value_to_set)
        elif type_key == self.TYPE_KEY_TEXT:
            self.set_text(value_to_set)
        else:
            raise Exception(f"Type key {type_key} is not supported.")
        return self

    def get_value_to_use_based_on_set_type(self) -> str:
        if self.type == self.TYPE_KEY_SSML:
            return self.ssml
        elif self.type == self.TYPE_KEY_TEXT:
            return self.text
        else:
            raise Exception(f"The set type_key was not one of the supported type keys.")


class Reprompt:
    def __init__(self):
        self.outputSpeech = OutputSpeech()


class Response:
    def __init__(self):
        self.outputSpeech = OutputSpeech()
        self.reprompt = Reprompt()
        self.card = Card()
        self._shouldEndSession = False

    @property
    def shouldEndSession(self):
        return self._shouldEndSession

    @shouldEndSession.setter
    def shouldEndSession(self, should_end_session: bool) -> None:
        if not isinstance(should_end_session, bool):
            raise Exception(f"should_end_session must be a bool object : {should_end_session}")
        self._shouldEndSession = should_end_session

    def to_platform_dict(self) -> dict:
        if CurrentPlatformData.used_platform_id == RefsAvailablePlatforms.REF_PLATFORM_ALEXA_V1:
            from inoft_vocal_framework.platforms_handlers.alexa_v1 import response
            output_response = response.Response()

            if self.outputSpeech.type == self.outputSpeech.TYPE_KEY_SSML:
                output_response.outputSpeech.set_ssml(self.outputSpeech.ssml)
            elif self.outputSpeech.type == self.outputSpeech.TYPE_KEY_TEXT:
                output_response.outputSpeech.set_text(self.outputSpeech.text)

            output_response.reprompt.outputSpeech.set_text("yaaaazazazaza")

            output_response.shouldEndSession = self.shouldEndSession

            platform_adapted_response_dict = output_response.to_dict()
            print(f"Final platform adapted on Alexa-v1 : {platform_adapted_response_dict}")
            return platform_adapted_response_dict

        elif CurrentPlatformData.used_platform_id == RefsAvailablePlatforms.REF_PLATFORM_GOOGLE_ASSISTANT_V1:
            from inoft_vocal_framework.platforms_handlers.dialogflow_v1.factories import response
            output_response = response.Payload()

            response_item = response.SimpleResponse()
            response_item.textToSpeech = self.outputSpeech.get_value_to_use_based_on_set_type()
            response_item.displayText = "Yaaaaaazaaaa !!!!!!!"
            print(f"the dddddidiidct = {response_item.to_json_dict()}")
            output_response.google.richResponse.add_response_item(response_item)

            output_response.google.expectUserResponse = True if not self.shouldEndSession else False

            platform_adapted_response_dict = output_response.to_dict()
            print(f"Final platform adapted on Google-Assistant-v1 : {platform_adapted_response_dict}")
            return platform_adapted_response_dict
        else:
            raise Exception(f"Platform with id {CurrentPlatformData.used_platform_id} is not supported.")
