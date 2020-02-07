from json import dumps as json_dumps

from inoft_vocal_framework.platforms_handlers.current_platform_static_data import CurrentPlatformData


class LambdaResponseWrapper:
    def __init__(self, response_dict: dict):
        if not isinstance(response_dict, dict):
            raise Exception(f"The response_dict must be of type dict and is of type : {type(response_dict)}")

        self._response_dict = response_dict

    def get_wrapped(self) -> dict:
        if CurrentPlatformData.is_alexa_v1 is True:
            return self._response_dict

        elif CurrentPlatformData.is_dialogflow_v1 is True:
            return {
                "isBase64Encoded": False,
                "statusCode": 200,
                "headers": dict(),
                "body": json_dumps(self._response_dict)
            }
