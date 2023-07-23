#include "QuickjsHelper.h"
#include "cutils.h"
#include "quickjs-libc.h"
#include <stdlib.h>

int QuickjsHelper_evalFile(JSContext *ctx, const char *filename, int module)
{
    uint8_t *buf;
    int ret, eval_flags;
    size_t buf_len;

    buf = js_load_file(ctx, &buf_len, filename);
    if (!buf)
    {
        perror(filename);
        exit(1);
    }

    if (module < 0)
    {
        module = (has_suffix(filename, ".mjs") || JS_DetectModule((const char *)buf, buf_len));
    }
    if (module)
    {
        eval_flags = JS_EVAL_TYPE_MODULE;
    }
    else
    {
        eval_flags = JS_EVAL_TYPE_GLOBAL;
    }
    ret = QuickjsHelper_evalBuffer(ctx, buf, buf_len, filename, eval_flags);
    js_free(ctx, buf);
    return ret;
}

int QuickjsHelper_evalBuffer(JSContext *ctx, const void *buffer, int bufferLength, const char *filename, int evalFlags)
{
    JSValue val;
    int ret;

    if ((evalFlags & JS_EVAL_TYPE_MASK) == JS_EVAL_TYPE_MODULE)
    {
        val = JS_Eval(ctx, (const char *)buffer, bufferLength, filename, evalFlags | JS_EVAL_FLAG_COMPILE_ONLY);
        if (!JS_IsException(val))
        {
            js_module_set_import_meta(ctx, val, TRUE, TRUE);
            val = JS_EvalFunction(ctx, val);
        }
    }
    else
    {
        val = JS_Eval(ctx, (const char *)buffer, bufferLength, filename, evalFlags);
    }
    if (JS_IsException(val))
    {
        js_std_dump_error(ctx);
        ret = -1;
    }
    else
    {
        ret = 0;
    }
    JS_FreeValue(ctx, val);
    return ret;
}

int QuickJS_ValueGetTag(JSValue v)
{
    return JS_VALUE_GET_TAG(v);
}

void QuickJS_FreeValue(JSContext *ctx, JSValue v)
{
    JS_FreeValue(ctx, v);
}

void QuickJS_FreeValueRT(JSRuntime *rt, JSValue v)
{
    return JS_FreeValueRT(rt, v);
}

void QuickJS_DupValue(JSContext *ctx, JSValue v)
{
    JS_DupValue(ctx, v);
}

JSValue QuickJS_DupValueRT(JSRuntime *rt, JSValueConst v)
{
    return JS_DupValueRT(rt, v);
}

JSValue QuickJS_NewFloat64(JSContext *ctx, double d)
{
    return JS_NewFloat64(ctx, d);
}

JSValue QuickJS_NewInt32(JSContext *ctx, int32_t val)
{
    return JS_NewInt32(ctx, val);
}

JSValue QuickJS_NewInt64(JSContext *ctx, int64_t val)
{
    return JS_NewInt64(ctx, val);
}

JSValue QuickJS_NewBool(JSContext *ctx, JS_BOOL val)
{
    return JS_NewBool(ctx, val);
}

JS_BOOL QuickJS_VALUE_IS_NAN(JSValue v)
{
    return JS_VALUE_IS_NAN(v);
}

double QuickJS_VALUE_GET_FLOAT64(JSValue v)
{
    return JS_VALUE_GET_FLOAT64(v);
}

int QuickJS_VALUE_GET_NORM_TAG(JSValue v)
{
    return JS_VALUE_GET_NORM_TAG(v);
}

JS_BOOL QuickJS_IsNumber(JSValueConst v)
{
    return JS_IsNumber(v);
}

JS_BOOL QuickJS_IsBigInt(JSContext *ctx, JSValueConst v)
{
    return JS_IsBigInt(ctx, v);
}

JS_BOOL QuickJS_IsBigFloat(JSValueConst v)
{
    return JS_IsBigFloat(v);
}

JS_BOOL QuickJS_IsBigDecimal(JSValueConst v)
{
    return JS_IsBigDecimal(v);
}

JS_BOOL QuickJS_IsBool(JSValueConst v)
{
    return JS_IsBool(v);
}

JS_BOOL QuickJS_IsNull(JSValueConst v)
{
    return JS_IsNull(v);
}

JS_BOOL QuickJS_IsUndefined(JSValueConst v)
{
    return JS_IsUndefined(v);
}

JS_BOOL QuickJS_IsException(JSValueConst v)
{
    return JS_IsException(v);
}

JS_BOOL QuickJS_IsUninitialized(JSValueConst v)
{
    return JS_IsUninitialized(v);
}

JS_BOOL QuickJS_IsString(JSValueConst v)
{
    return JS_IsString(v);
}

JS_BOOL QuickJS_IsSymbol(JSValueConst v)
{
    return JS_IsSymbol(v);
}

JS_BOOL QuickJS_IsObject(JSValueConst v)
{
    return JS_IsObject(v);
}

int QuickJS_ToUint32(JSContext *ctx, uint32_t *pres, JSValueConst val)
{
    return JS_ToUint32(ctx, pres, val);
}

int QuickJS_SetProperty(JSContext *ctx, JSValueConst this_obj, JSAtom prop, JSValue val)
{
    return JS_SetProperty(ctx, this_obj, prop, val);
}

JSValue QuickJS_NewCFunction(JSContext *ctx, JSCFunction *func, const char *name, int length)
{
    return JS_NewCFunction(ctx, func, name, length);
}

JSValue QuickJS_NewCFunctionMagic(JSContext *ctx, JSCFunctionMagic *func, const char *name, int length,
                                  JSCFunctionEnum cproto, int magic)
{
    return JS_NewCFunctionMagic(ctx, func, name, length, cproto, magic);
}

JSValue QuickJS_MKVAL(int tag, int val)
{
    return JS_MKVAL(tag, val);
}

JSValue QuickJS_NULL()
{
    return JS_MKVAL(JS_TAG_NULL, 0);
}

JSValue QuickJS_UNDEFINED()
{
    return JS_MKVAL(JS_TAG_UNDEFINED, 0);
}

JSValue QuickJS_FALSE()
{
    return JS_MKVAL(JS_TAG_BOOL, 0);
}

JSValue QuickJS_TRUE()
{
    return JS_MKVAL(JS_TAG_BOOL, 1);
}

JSValue QuickJS_EXCEPTION()
{
    return JS_MKVAL(JS_TAG_EXCEPTION, 0);
}

JSValue QuickJS_UNINITIALIZED()
{
    return JS_MKVAL(JS_TAG_UNINITIALIZED, 0);
}