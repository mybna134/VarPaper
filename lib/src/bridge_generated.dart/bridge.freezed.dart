// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'bridge.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$ServiceEvent {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ServiceEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'ServiceEvent()';
}


}

/// @nodoc
class $ServiceEventCopyWith<$Res>  {
$ServiceEventCopyWith(ServiceEvent _, $Res Function(ServiceEvent) __);
}


/// Adds pattern-matching-related methods to [ServiceEvent].
extension ServiceEventPatterns on ServiceEvent {
/// A variant of `map` that fallback to returning `orElse`.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case _:
///     return orElse();
/// }
/// ```

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( ServiceEvent_EngineStarted value)?  engineStarted,TResult Function( ServiceEvent_EngineStopped value)?  engineStopped,TResult Function( ServiceEvent_WallpaperApplied value)?  wallpaperApplied,TResult Function( ServiceEvent_WallpaperCleared value)?  wallpaperCleared,TResult Function( ServiceEvent_OutputsChanged value)?  outputsChanged,TResult Function( ServiceEvent_Error value)?  error,required TResult orElse(),}){
final _that = this;
switch (_that) {
case ServiceEvent_EngineStarted() when engineStarted != null:
return engineStarted(_that);case ServiceEvent_EngineStopped() when engineStopped != null:
return engineStopped(_that);case ServiceEvent_WallpaperApplied() when wallpaperApplied != null:
return wallpaperApplied(_that);case ServiceEvent_WallpaperCleared() when wallpaperCleared != null:
return wallpaperCleared(_that);case ServiceEvent_OutputsChanged() when outputsChanged != null:
return outputsChanged(_that);case ServiceEvent_Error() when error != null:
return error(_that);case _:
  return orElse();

}
}
/// A `switch`-like method, using callbacks.
///
/// Callbacks receives the raw object, upcasted.
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case final Subclass2 value:
///     return ...;
/// }
/// ```

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( ServiceEvent_EngineStarted value)  engineStarted,required TResult Function( ServiceEvent_EngineStopped value)  engineStopped,required TResult Function( ServiceEvent_WallpaperApplied value)  wallpaperApplied,required TResult Function( ServiceEvent_WallpaperCleared value)  wallpaperCleared,required TResult Function( ServiceEvent_OutputsChanged value)  outputsChanged,required TResult Function( ServiceEvent_Error value)  error,}){
final _that = this;
switch (_that) {
case ServiceEvent_EngineStarted():
return engineStarted(_that);case ServiceEvent_EngineStopped():
return engineStopped(_that);case ServiceEvent_WallpaperApplied():
return wallpaperApplied(_that);case ServiceEvent_WallpaperCleared():
return wallpaperCleared(_that);case ServiceEvent_OutputsChanged():
return outputsChanged(_that);case ServiceEvent_Error():
return error(_that);}
}
/// A variant of `map` that fallback to returning `null`.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case _:
///     return null;
/// }
/// ```

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( ServiceEvent_EngineStarted value)?  engineStarted,TResult? Function( ServiceEvent_EngineStopped value)?  engineStopped,TResult? Function( ServiceEvent_WallpaperApplied value)?  wallpaperApplied,TResult? Function( ServiceEvent_WallpaperCleared value)?  wallpaperCleared,TResult? Function( ServiceEvent_OutputsChanged value)?  outputsChanged,TResult? Function( ServiceEvent_Error value)?  error,}){
final _that = this;
switch (_that) {
case ServiceEvent_EngineStarted() when engineStarted != null:
return engineStarted(_that);case ServiceEvent_EngineStopped() when engineStopped != null:
return engineStopped(_that);case ServiceEvent_WallpaperApplied() when wallpaperApplied != null:
return wallpaperApplied(_that);case ServiceEvent_WallpaperCleared() when wallpaperCleared != null:
return wallpaperCleared(_that);case ServiceEvent_OutputsChanged() when outputsChanged != null:
return outputsChanged(_that);case ServiceEvent_Error() when error != null:
return error(_that);case _:
  return null;

}
}
/// A variant of `when` that fallback to an `orElse` callback.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case _:
///     return orElse();
/// }
/// ```

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function()?  engineStarted,TResult Function()?  engineStopped,TResult Function( String output,  String path)?  wallpaperApplied,TResult Function( String output)?  wallpaperCleared,TResult Function( List<MonitorDto> outputs)?  outputsChanged,TResult Function( String code,  String message)?  error,required TResult orElse(),}) {final _that = this;
switch (_that) {
case ServiceEvent_EngineStarted() when engineStarted != null:
return engineStarted();case ServiceEvent_EngineStopped() when engineStopped != null:
return engineStopped();case ServiceEvent_WallpaperApplied() when wallpaperApplied != null:
return wallpaperApplied(_that.output,_that.path);case ServiceEvent_WallpaperCleared() when wallpaperCleared != null:
return wallpaperCleared(_that.output);case ServiceEvent_OutputsChanged() when outputsChanged != null:
return outputsChanged(_that.outputs);case ServiceEvent_Error() when error != null:
return error(_that.code,_that.message);case _:
  return orElse();

}
}
/// A `switch`-like method, using callbacks.
///
/// As opposed to `map`, this offers destructuring.
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case Subclass2(:final field2):
///     return ...;
/// }
/// ```

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function()  engineStarted,required TResult Function()  engineStopped,required TResult Function( String output,  String path)  wallpaperApplied,required TResult Function( String output)  wallpaperCleared,required TResult Function( List<MonitorDto> outputs)  outputsChanged,required TResult Function( String code,  String message)  error,}) {final _that = this;
switch (_that) {
case ServiceEvent_EngineStarted():
return engineStarted();case ServiceEvent_EngineStopped():
return engineStopped();case ServiceEvent_WallpaperApplied():
return wallpaperApplied(_that.output,_that.path);case ServiceEvent_WallpaperCleared():
return wallpaperCleared(_that.output);case ServiceEvent_OutputsChanged():
return outputsChanged(_that.outputs);case ServiceEvent_Error():
return error(_that.code,_that.message);}
}
/// A variant of `when` that fallback to returning `null`
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case _:
///     return null;
/// }
/// ```

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function()?  engineStarted,TResult? Function()?  engineStopped,TResult? Function( String output,  String path)?  wallpaperApplied,TResult? Function( String output)?  wallpaperCleared,TResult? Function( List<MonitorDto> outputs)?  outputsChanged,TResult? Function( String code,  String message)?  error,}) {final _that = this;
switch (_that) {
case ServiceEvent_EngineStarted() when engineStarted != null:
return engineStarted();case ServiceEvent_EngineStopped() when engineStopped != null:
return engineStopped();case ServiceEvent_WallpaperApplied() when wallpaperApplied != null:
return wallpaperApplied(_that.output,_that.path);case ServiceEvent_WallpaperCleared() when wallpaperCleared != null:
return wallpaperCleared(_that.output);case ServiceEvent_OutputsChanged() when outputsChanged != null:
return outputsChanged(_that.outputs);case ServiceEvent_Error() when error != null:
return error(_that.code,_that.message);case _:
  return null;

}
}

}

/// @nodoc


class ServiceEvent_EngineStarted extends ServiceEvent {
  const ServiceEvent_EngineStarted(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ServiceEvent_EngineStarted);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'ServiceEvent.engineStarted()';
}


}




/// @nodoc


class ServiceEvent_EngineStopped extends ServiceEvent {
  const ServiceEvent_EngineStopped(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ServiceEvent_EngineStopped);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'ServiceEvent.engineStopped()';
}


}




/// @nodoc


class ServiceEvent_WallpaperApplied extends ServiceEvent {
  const ServiceEvent_WallpaperApplied({required this.output, required this.path}): super._();
  

 final  String output;
 final  String path;

/// Create a copy of ServiceEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$ServiceEvent_WallpaperAppliedCopyWith<ServiceEvent_WallpaperApplied> get copyWith => _$ServiceEvent_WallpaperAppliedCopyWithImpl<ServiceEvent_WallpaperApplied>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ServiceEvent_WallpaperApplied&&(identical(other.output, output) || other.output == output)&&(identical(other.path, path) || other.path == path));
}


@override
int get hashCode => Object.hash(runtimeType,output,path);

@override
String toString() {
  return 'ServiceEvent.wallpaperApplied(output: $output, path: $path)';
}


}

/// @nodoc
abstract mixin class $ServiceEvent_WallpaperAppliedCopyWith<$Res> implements $ServiceEventCopyWith<$Res> {
  factory $ServiceEvent_WallpaperAppliedCopyWith(ServiceEvent_WallpaperApplied value, $Res Function(ServiceEvent_WallpaperApplied) _then) = _$ServiceEvent_WallpaperAppliedCopyWithImpl;
@useResult
$Res call({
 String output, String path
});




}
/// @nodoc
class _$ServiceEvent_WallpaperAppliedCopyWithImpl<$Res>
    implements $ServiceEvent_WallpaperAppliedCopyWith<$Res> {
  _$ServiceEvent_WallpaperAppliedCopyWithImpl(this._self, this._then);

  final ServiceEvent_WallpaperApplied _self;
  final $Res Function(ServiceEvent_WallpaperApplied) _then;

/// Create a copy of ServiceEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? output = null,Object? path = null,}) {
  return _then(ServiceEvent_WallpaperApplied(
output: null == output ? _self.output : output // ignore: cast_nullable_to_non_nullable
as String,path: null == path ? _self.path : path // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class ServiceEvent_WallpaperCleared extends ServiceEvent {
  const ServiceEvent_WallpaperCleared({required this.output}): super._();
  

 final  String output;

/// Create a copy of ServiceEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$ServiceEvent_WallpaperClearedCopyWith<ServiceEvent_WallpaperCleared> get copyWith => _$ServiceEvent_WallpaperClearedCopyWithImpl<ServiceEvent_WallpaperCleared>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ServiceEvent_WallpaperCleared&&(identical(other.output, output) || other.output == output));
}


@override
int get hashCode => Object.hash(runtimeType,output);

@override
String toString() {
  return 'ServiceEvent.wallpaperCleared(output: $output)';
}


}

/// @nodoc
abstract mixin class $ServiceEvent_WallpaperClearedCopyWith<$Res> implements $ServiceEventCopyWith<$Res> {
  factory $ServiceEvent_WallpaperClearedCopyWith(ServiceEvent_WallpaperCleared value, $Res Function(ServiceEvent_WallpaperCleared) _then) = _$ServiceEvent_WallpaperClearedCopyWithImpl;
@useResult
$Res call({
 String output
});




}
/// @nodoc
class _$ServiceEvent_WallpaperClearedCopyWithImpl<$Res>
    implements $ServiceEvent_WallpaperClearedCopyWith<$Res> {
  _$ServiceEvent_WallpaperClearedCopyWithImpl(this._self, this._then);

  final ServiceEvent_WallpaperCleared _self;
  final $Res Function(ServiceEvent_WallpaperCleared) _then;

/// Create a copy of ServiceEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? output = null,}) {
  return _then(ServiceEvent_WallpaperCleared(
output: null == output ? _self.output : output // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class ServiceEvent_OutputsChanged extends ServiceEvent {
  const ServiceEvent_OutputsChanged({required  List<MonitorDto> outputs}): _outputs = outputs,super._();
  

 final  List<MonitorDto> _outputs;
 List<MonitorDto> get outputs {
  if (_outputs is EqualUnmodifiableListView) return _outputs;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_outputs);
}


/// Create a copy of ServiceEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$ServiceEvent_OutputsChangedCopyWith<ServiceEvent_OutputsChanged> get copyWith => _$ServiceEvent_OutputsChangedCopyWithImpl<ServiceEvent_OutputsChanged>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ServiceEvent_OutputsChanged&&const DeepCollectionEquality().equals(other._outputs, _outputs));
}


@override
int get hashCode => Object.hash(runtimeType,const DeepCollectionEquality().hash(_outputs));

@override
String toString() {
  return 'ServiceEvent.outputsChanged(outputs: $outputs)';
}


}

/// @nodoc
abstract mixin class $ServiceEvent_OutputsChangedCopyWith<$Res> implements $ServiceEventCopyWith<$Res> {
  factory $ServiceEvent_OutputsChangedCopyWith(ServiceEvent_OutputsChanged value, $Res Function(ServiceEvent_OutputsChanged) _then) = _$ServiceEvent_OutputsChangedCopyWithImpl;
@useResult
$Res call({
 List<MonitorDto> outputs
});




}
/// @nodoc
class _$ServiceEvent_OutputsChangedCopyWithImpl<$Res>
    implements $ServiceEvent_OutputsChangedCopyWith<$Res> {
  _$ServiceEvent_OutputsChangedCopyWithImpl(this._self, this._then);

  final ServiceEvent_OutputsChanged _self;
  final $Res Function(ServiceEvent_OutputsChanged) _then;

/// Create a copy of ServiceEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? outputs = null,}) {
  return _then(ServiceEvent_OutputsChanged(
outputs: null == outputs ? _self._outputs : outputs // ignore: cast_nullable_to_non_nullable
as List<MonitorDto>,
  ));
}


}

/// @nodoc


class ServiceEvent_Error extends ServiceEvent {
  const ServiceEvent_Error({required this.code, required this.message}): super._();
  

 final  String code;
 final  String message;

/// Create a copy of ServiceEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$ServiceEvent_ErrorCopyWith<ServiceEvent_Error> get copyWith => _$ServiceEvent_ErrorCopyWithImpl<ServiceEvent_Error>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ServiceEvent_Error&&(identical(other.code, code) || other.code == code)&&(identical(other.message, message) || other.message == message));
}


@override
int get hashCode => Object.hash(runtimeType,code,message);

@override
String toString() {
  return 'ServiceEvent.error(code: $code, message: $message)';
}


}

/// @nodoc
abstract mixin class $ServiceEvent_ErrorCopyWith<$Res> implements $ServiceEventCopyWith<$Res> {
  factory $ServiceEvent_ErrorCopyWith(ServiceEvent_Error value, $Res Function(ServiceEvent_Error) _then) = _$ServiceEvent_ErrorCopyWithImpl;
@useResult
$Res call({
 String code, String message
});




}
/// @nodoc
class _$ServiceEvent_ErrorCopyWithImpl<$Res>
    implements $ServiceEvent_ErrorCopyWith<$Res> {
  _$ServiceEvent_ErrorCopyWithImpl(this._self, this._then);

  final ServiceEvent_Error _self;
  final $Res Function(ServiceEvent_Error) _then;

/// Create a copy of ServiceEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? code = null,Object? message = null,}) {
  return _then(ServiceEvent_Error(
code: null == code ? _self.code : code // ignore: cast_nullable_to_non_nullable
as String,message: null == message ? _self.message : message // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

// dart format on
