// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'settings_store.dart';

// **************************************************************************
// IsarCollectionGenerator
// **************************************************************************

// coverage:ignore-file
// ignore_for_file: duplicate_ignore, non_constant_identifier_names, constant_identifier_names, invalid_use_of_protected_member, unnecessary_cast, prefer_const_constructors, lines_longer_than_80_chars, require_trailing_commas, inference_failure_on_function_invocation, unnecessary_parenthesis, unnecessary_raw_strings, unnecessary_null_checks, join_return_with_assignment, prefer_final_locals, avoid_js_rounded_ints, avoid_positional_boolean_parameters, always_specify_types

extension GetAppSettingsRecordCollection on Isar {
  IsarCollection<AppSettingsRecord> get appSettingsRecords => this.collection();
}

const AppSettingsRecordSchema = CollectionSchema(
  name: r'AppSettingsRecord',
  id: -5800169138830006153,
  properties: {
    r'autostartEnabled': PropertySchema(
      id: 0,
      name: r'autostartEnabled',
      type: IsarType.bool,
    ),
    r'batteryFpsLimit': PropertySchema(
      id: 1,
      name: r'batteryFpsLimit',
      type: IsarType.long,
    ),
    r'colorSchemeVariant': PropertySchema(
      id: 2,
      name: r'colorSchemeVariant',
      type: IsarType.string,
    ),
    r'customThemeColors': PropertySchema(
      id: 3,
      name: r'customThemeColors',
      type: IsarType.longList,
    ),
    r'detailPanelVisible': PropertySchema(
      id: 4,
      name: r'detailPanelVisible',
      type: IsarType.bool,
    ),
    r'fpsLimit': PropertySchema(id: 5, name: r'fpsLimit', type: IsarType.long),
    r'hdrMode': PropertySchema(id: 6, name: r'hdrMode', type: IsarType.string),
    r'hwdec': PropertySchema(id: 7, name: r'hwdec', type: IsarType.string),
    r'language': PropertySchema(
      id: 8,
      name: r'language',
      type: IsarType.string,
    ),
    r'layout': PropertySchema(id: 9, name: r'layout', type: IsarType.string),
    r'libraryFolders': PropertySchema(
      id: 10,
      name: r'libraryFolders',
      type: IsarType.stringList,
    ),
    r'loopMode': PropertySchema(id: 11, name: r'loopMode', type: IsarType.bool),
    r'minimizeToTray': PropertySchema(
      id: 12,
      name: r'minimizeToTray',
      type: IsarType.bool,
    ),
    r'mute': PropertySchema(id: 13, name: r'mute', type: IsarType.bool),
    r'pauseOnBattery': PropertySchema(
      id: 14,
      name: r'pauseOnBattery',
      type: IsarType.bool,
    ),
    r'pauseOnFullscreen': PropertySchema(
      id: 15,
      name: r'pauseOnFullscreen',
      type: IsarType.bool,
    ),
    r'playbackRate': PropertySchema(
      id: 16,
      name: r'playbackRate',
      type: IsarType.double,
    ),
    r'preferredMonitor': PropertySchema(
      id: 17,
      name: r'preferredMonitor',
      type: IsarType.string,
    ),
    r'renderer': PropertySchema(
      id: 18,
      name: r'renderer',
      type: IsarType.string,
    ),
    r'restoreLastWallpaper': PropertySchema(
      id: 19,
      name: r'restoreLastWallpaper',
      type: IsarType.bool,
    ),
    r'shuffle': PropertySchema(id: 20, name: r'shuffle', type: IsarType.bool),
    r'sidebarCollapsed': PropertySchema(
      id: 21,
      name: r'sidebarCollapsed',
      type: IsarType.bool,
    ),
    r'startMinimized': PropertySchema(
      id: 22,
      name: r'startMinimized',
      type: IsarType.bool,
    ),
    r'startTime': PropertySchema(
      id: 23,
      name: r'startTime',
      type: IsarType.double,
    ),
    r'theme': PropertySchema(id: 24, name: r'theme', type: IsarType.string),
    r'themeColor': PropertySchema(
      id: 25,
      name: r'themeColor',
      type: IsarType.long,
    ),
    r'toneMappingAlgorithm': PropertySchema(
      id: 26,
      name: r'toneMappingAlgorithm',
      type: IsarType.string,
    ),
    r'toneMappingComputePeak': PropertySchema(
      id: 27,
      name: r'toneMappingComputePeak',
      type: IsarType.bool,
    ),
    r'toneMappingMode': PropertySchema(
      id: 28,
      name: r'toneMappingMode',
      type: IsarType.string,
    ),
    r'toneMappingParam': PropertySchema(
      id: 29,
      name: r'toneMappingParam',
      type: IsarType.double,
    ),
    r'volume': PropertySchema(id: 30, name: r'volume', type: IsarType.double),
    r'windowHeight': PropertySchema(
      id: 31,
      name: r'windowHeight',
      type: IsarType.long,
    ),
    r'windowWidth': PropertySchema(
      id: 32,
      name: r'windowWidth',
      type: IsarType.long,
    ),
  },

  estimateSize: _appSettingsRecordEstimateSize,
  serialize: _appSettingsRecordSerialize,
  deserialize: _appSettingsRecordDeserialize,
  deserializeProp: _appSettingsRecordDeserializeProp,
  idName: r'id',
  indexes: {},
  links: {},
  embeddedSchemas: {},

  getId: _appSettingsRecordGetId,
  getLinks: _appSettingsRecordGetLinks,
  attach: _appSettingsRecordAttach,
  version: '3.3.2',
);

int _appSettingsRecordEstimateSize(
  AppSettingsRecord object,
  List<int> offsets,
  Map<Type, List<int>> allOffsets,
) {
  var bytesCount = offsets.last;
  bytesCount += 3 + object.colorSchemeVariant.length * 3;
  bytesCount += 3 + object.customThemeColors.length * 8;
  bytesCount += 3 + object.hdrMode.length * 3;
  bytesCount += 3 + object.hwdec.length * 3;
  bytesCount += 3 + object.language.length * 3;
  bytesCount += 3 + object.layout.length * 3;
  bytesCount += 3 + object.libraryFolders.length * 3;
  {
    for (var i = 0; i < object.libraryFolders.length; i++) {
      final value = object.libraryFolders[i];
      bytesCount += value.length * 3;
    }
  }
  {
    final value = object.preferredMonitor;
    if (value != null) {
      bytesCount += 3 + value.length * 3;
    }
  }
  bytesCount += 3 + object.renderer.length * 3;
  bytesCount += 3 + object.theme.length * 3;
  bytesCount += 3 + object.toneMappingAlgorithm.length * 3;
  bytesCount += 3 + object.toneMappingMode.length * 3;
  return bytesCount;
}

void _appSettingsRecordSerialize(
  AppSettingsRecord object,
  IsarWriter writer,
  List<int> offsets,
  Map<Type, List<int>> allOffsets,
) {
  writer.writeBool(offsets[0], object.autostartEnabled);
  writer.writeLong(offsets[1], object.batteryFpsLimit);
  writer.writeString(offsets[2], object.colorSchemeVariant);
  writer.writeLongList(offsets[3], object.customThemeColors);
  writer.writeBool(offsets[4], object.detailPanelVisible);
  writer.writeLong(offsets[5], object.fpsLimit);
  writer.writeString(offsets[6], object.hdrMode);
  writer.writeString(offsets[7], object.hwdec);
  writer.writeString(offsets[8], object.language);
  writer.writeString(offsets[9], object.layout);
  writer.writeStringList(offsets[10], object.libraryFolders);
  writer.writeBool(offsets[11], object.loopMode);
  writer.writeBool(offsets[12], object.minimizeToTray);
  writer.writeBool(offsets[13], object.mute);
  writer.writeBool(offsets[14], object.pauseOnBattery);
  writer.writeBool(offsets[15], object.pauseOnFullscreen);
  writer.writeDouble(offsets[16], object.playbackRate);
  writer.writeString(offsets[17], object.preferredMonitor);
  writer.writeString(offsets[18], object.renderer);
  writer.writeBool(offsets[19], object.restoreLastWallpaper);
  writer.writeBool(offsets[20], object.shuffle);
  writer.writeBool(offsets[21], object.sidebarCollapsed);
  writer.writeBool(offsets[22], object.startMinimized);
  writer.writeDouble(offsets[23], object.startTime);
  writer.writeString(offsets[24], object.theme);
  writer.writeLong(offsets[25], object.themeColor);
  writer.writeString(offsets[26], object.toneMappingAlgorithm);
  writer.writeBool(offsets[27], object.toneMappingComputePeak);
  writer.writeString(offsets[28], object.toneMappingMode);
  writer.writeDouble(offsets[29], object.toneMappingParam);
  writer.writeDouble(offsets[30], object.volume);
  writer.writeLong(offsets[31], object.windowHeight);
  writer.writeLong(offsets[32], object.windowWidth);
}

AppSettingsRecord _appSettingsRecordDeserialize(
  Id id,
  IsarReader reader,
  List<int> offsets,
  Map<Type, List<int>> allOffsets,
) {
  final object = AppSettingsRecord();
  object.autostartEnabled = reader.readBool(offsets[0]);
  object.batteryFpsLimit = reader.readLongOrNull(offsets[1]);
  object.colorSchemeVariant = reader.readString(offsets[2]);
  object.customThemeColors = reader.readLongList(offsets[3]) ?? [];
  object.detailPanelVisible = reader.readBool(offsets[4]);
  object.fpsLimit = reader.readLongOrNull(offsets[5]);
  object.hdrMode = reader.readString(offsets[6]);
  object.hwdec = reader.readString(offsets[7]);
  object.id = id;
  object.language = reader.readString(offsets[8]);
  object.layout = reader.readString(offsets[9]);
  object.libraryFolders = reader.readStringList(offsets[10]) ?? [];
  object.loopMode = reader.readBool(offsets[11]);
  object.minimizeToTray = reader.readBool(offsets[12]);
  object.mute = reader.readBool(offsets[13]);
  object.pauseOnBattery = reader.readBool(offsets[14]);
  object.pauseOnFullscreen = reader.readBool(offsets[15]);
  object.playbackRate = reader.readDouble(offsets[16]);
  object.preferredMonitor = reader.readStringOrNull(offsets[17]);
  object.renderer = reader.readString(offsets[18]);
  object.restoreLastWallpaper = reader.readBool(offsets[19]);
  object.shuffle = reader.readBool(offsets[20]);
  object.sidebarCollapsed = reader.readBool(offsets[21]);
  object.startMinimized = reader.readBool(offsets[22]);
  object.startTime = reader.readDouble(offsets[23]);
  object.theme = reader.readString(offsets[24]);
  object.themeColor = reader.readLong(offsets[25]);
  object.toneMappingAlgorithm = reader.readString(offsets[26]);
  object.toneMappingComputePeak = reader.readBool(offsets[27]);
  object.toneMappingMode = reader.readString(offsets[28]);
  object.toneMappingParam = reader.readDouble(offsets[29]);
  object.volume = reader.readDouble(offsets[30]);
  object.windowHeight = reader.readLong(offsets[31]);
  object.windowWidth = reader.readLong(offsets[32]);
  return object;
}

P _appSettingsRecordDeserializeProp<P>(
  IsarReader reader,
  int propertyId,
  int offset,
  Map<Type, List<int>> allOffsets,
) {
  switch (propertyId) {
    case 0:
      return (reader.readBool(offset)) as P;
    case 1:
      return (reader.readLongOrNull(offset)) as P;
    case 2:
      return (reader.readString(offset)) as P;
    case 3:
      return (reader.readLongList(offset) ?? []) as P;
    case 4:
      return (reader.readBool(offset)) as P;
    case 5:
      return (reader.readLongOrNull(offset)) as P;
    case 6:
      return (reader.readString(offset)) as P;
    case 7:
      return (reader.readString(offset)) as P;
    case 8:
      return (reader.readString(offset)) as P;
    case 9:
      return (reader.readString(offset)) as P;
    case 10:
      return (reader.readStringList(offset) ?? []) as P;
    case 11:
      return (reader.readBool(offset)) as P;
    case 12:
      return (reader.readBool(offset)) as P;
    case 13:
      return (reader.readBool(offset)) as P;
    case 14:
      return (reader.readBool(offset)) as P;
    case 15:
      return (reader.readBool(offset)) as P;
    case 16:
      return (reader.readDouble(offset)) as P;
    case 17:
      return (reader.readStringOrNull(offset)) as P;
    case 18:
      return (reader.readString(offset)) as P;
    case 19:
      return (reader.readBool(offset)) as P;
    case 20:
      return (reader.readBool(offset)) as P;
    case 21:
      return (reader.readBool(offset)) as P;
    case 22:
      return (reader.readBool(offset)) as P;
    case 23:
      return (reader.readDouble(offset)) as P;
    case 24:
      return (reader.readString(offset)) as P;
    case 25:
      return (reader.readLong(offset)) as P;
    case 26:
      return (reader.readString(offset)) as P;
    case 27:
      return (reader.readBool(offset)) as P;
    case 28:
      return (reader.readString(offset)) as P;
    case 29:
      return (reader.readDouble(offset)) as P;
    case 30:
      return (reader.readDouble(offset)) as P;
    case 31:
      return (reader.readLong(offset)) as P;
    case 32:
      return (reader.readLong(offset)) as P;
    default:
      throw IsarError('Unknown property with id $propertyId');
  }
}

Id _appSettingsRecordGetId(AppSettingsRecord object) {
  return object.id;
}

List<IsarLinkBase<dynamic>> _appSettingsRecordGetLinks(
  AppSettingsRecord object,
) {
  return [];
}

void _appSettingsRecordAttach(
  IsarCollection<dynamic> col,
  Id id,
  AppSettingsRecord object,
) {
  object.id = id;
}

extension AppSettingsRecordQueryWhereSort
    on QueryBuilder<AppSettingsRecord, AppSettingsRecord, QWhere> {
  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterWhere> anyId() {
    return QueryBuilder.apply(this, (query) {
      return query.addWhereClause(const IdWhereClause.any());
    });
  }
}

extension AppSettingsRecordQueryWhere
    on QueryBuilder<AppSettingsRecord, AppSettingsRecord, QWhereClause> {
  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterWhereClause>
  idEqualTo(Id id) {
    return QueryBuilder.apply(this, (query) {
      return query.addWhereClause(IdWhereClause.between(lower: id, upper: id));
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterWhereClause>
  idNotEqualTo(Id id) {
    return QueryBuilder.apply(this, (query) {
      if (query.whereSort == Sort.asc) {
        return query
            .addWhereClause(
              IdWhereClause.lessThan(upper: id, includeUpper: false),
            )
            .addWhereClause(
              IdWhereClause.greaterThan(lower: id, includeLower: false),
            );
      } else {
        return query
            .addWhereClause(
              IdWhereClause.greaterThan(lower: id, includeLower: false),
            )
            .addWhereClause(
              IdWhereClause.lessThan(upper: id, includeUpper: false),
            );
      }
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterWhereClause>
  idGreaterThan(Id id, {bool include = false}) {
    return QueryBuilder.apply(this, (query) {
      return query.addWhereClause(
        IdWhereClause.greaterThan(lower: id, includeLower: include),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterWhereClause>
  idLessThan(Id id, {bool include = false}) {
    return QueryBuilder.apply(this, (query) {
      return query.addWhereClause(
        IdWhereClause.lessThan(upper: id, includeUpper: include),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterWhereClause>
  idBetween(
    Id lowerId,
    Id upperId, {
    bool includeLower = true,
    bool includeUpper = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addWhereClause(
        IdWhereClause.between(
          lower: lowerId,
          includeLower: includeLower,
          upper: upperId,
          includeUpper: includeUpper,
        ),
      );
    });
  }
}

extension AppSettingsRecordQueryFilter
    on QueryBuilder<AppSettingsRecord, AppSettingsRecord, QFilterCondition> {
  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  autostartEnabledEqualTo(bool value) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.equalTo(property: r'autostartEnabled', value: value),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  batteryFpsLimitIsNull() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        const FilterCondition.isNull(property: r'batteryFpsLimit'),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  batteryFpsLimitIsNotNull() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        const FilterCondition.isNotNull(property: r'batteryFpsLimit'),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  batteryFpsLimitEqualTo(int? value) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.equalTo(property: r'batteryFpsLimit', value: value),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  batteryFpsLimitGreaterThan(int? value, {bool include = false}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.greaterThan(
          include: include,
          property: r'batteryFpsLimit',
          value: value,
        ),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  batteryFpsLimitLessThan(int? value, {bool include = false}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.lessThan(
          include: include,
          property: r'batteryFpsLimit',
          value: value,
        ),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  batteryFpsLimitBetween(
    int? lower,
    int? upper, {
    bool includeLower = true,
    bool includeUpper = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.between(
          property: r'batteryFpsLimit',
          lower: lower,
          includeLower: includeLower,
          upper: upper,
          includeUpper: includeUpper,
        ),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  colorSchemeVariantEqualTo(String value, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.equalTo(
          property: r'colorSchemeVariant',
          value: value,
          caseSensitive: caseSensitive,
        ),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  colorSchemeVariantGreaterThan(
    String value, {
    bool include = false,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.greaterThan(
          include: include,
          property: r'colorSchemeVariant',
          value: value,
          caseSensitive: caseSensitive,
        ),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  colorSchemeVariantLessThan(
    String value, {
    bool include = false,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.lessThan(
          include: include,
          property: r'colorSchemeVariant',
          value: value,
          caseSensitive: caseSensitive,
        ),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  colorSchemeVariantBetween(
    String lower,
    String upper, {
    bool includeLower = true,
    bool includeUpper = true,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.between(
          property: r'colorSchemeVariant',
          lower: lower,
          includeLower: includeLower,
          upper: upper,
          includeUpper: includeUpper,
          caseSensitive: caseSensitive,
        ),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  colorSchemeVariantStartsWith(String value, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.startsWith(
          property: r'colorSchemeVariant',
          value: value,
          caseSensitive: caseSensitive,
        ),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  colorSchemeVariantEndsWith(String value, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.endsWith(
          property: r'colorSchemeVariant',
          value: value,
          caseSensitive: caseSensitive,
        ),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  colorSchemeVariantContains(String value, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.contains(
          property: r'colorSchemeVariant',
          value: value,
          caseSensitive: caseSensitive,
        ),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  colorSchemeVariantMatches(String pattern, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.matches(
          property: r'colorSchemeVariant',
          wildcard: pattern,
          caseSensitive: caseSensitive,
        ),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  colorSchemeVariantIsEmpty() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.equalTo(property: r'colorSchemeVariant', value: ''),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  colorSchemeVariantIsNotEmpty() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.greaterThan(property: r'colorSchemeVariant', value: ''),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  customThemeColorsElementEqualTo(int value) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.equalTo(property: r'customThemeColors', value: value),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  customThemeColorsElementGreaterThan(int value, {bool include = false}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.greaterThan(
          include: include,
          property: r'customThemeColors',
          value: value,
        ),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  customThemeColorsElementLessThan(int value, {bool include = false}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.lessThan(
          include: include,
          property: r'customThemeColors',
          value: value,
        ),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  customThemeColorsElementBetween(
    int lower,
    int upper, {
    bool includeLower = true,
    bool includeUpper = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.between(
          property: r'customThemeColors',
          lower: lower,
          includeLower: includeLower,
          upper: upper,
          includeUpper: includeUpper,
        ),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  customThemeColorsLengthEqualTo(int length) {
    return QueryBuilder.apply(this, (query) {
      return query.listLength(r'customThemeColors', length, true, length, true);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  customThemeColorsIsEmpty() {
    return QueryBuilder.apply(this, (query) {
      return query.listLength(r'customThemeColors', 0, true, 0, true);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  customThemeColorsIsNotEmpty() {
    return QueryBuilder.apply(this, (query) {
      return query.listLength(r'customThemeColors', 0, false, 999999, true);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  customThemeColorsLengthLessThan(int length, {bool include = false}) {
    return QueryBuilder.apply(this, (query) {
      return query.listLength(r'customThemeColors', 0, true, length, include);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  customThemeColorsLengthGreaterThan(int length, {bool include = false}) {
    return QueryBuilder.apply(this, (query) {
      return query.listLength(
        r'customThemeColors',
        length,
        include,
        999999,
        true,
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  customThemeColorsLengthBetween(
    int lower,
    int upper, {
    bool includeLower = true,
    bool includeUpper = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.listLength(
        r'customThemeColors',
        lower,
        includeLower,
        upper,
        includeUpper,
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  detailPanelVisibleEqualTo(bool value) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.equalTo(property: r'detailPanelVisible', value: value),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  fpsLimitIsNull() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        const FilterCondition.isNull(property: r'fpsLimit'),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  fpsLimitIsNotNull() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        const FilterCondition.isNotNull(property: r'fpsLimit'),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  fpsLimitEqualTo(int? value) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.equalTo(property: r'fpsLimit', value: value),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  fpsLimitGreaterThan(int? value, {bool include = false}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.greaterThan(
          include: include,
          property: r'fpsLimit',
          value: value,
        ),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  fpsLimitLessThan(int? value, {bool include = false}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.lessThan(
          include: include,
          property: r'fpsLimit',
          value: value,
        ),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  fpsLimitBetween(
    int? lower,
    int? upper, {
    bool includeLower = true,
    bool includeUpper = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.between(
          property: r'fpsLimit',
          lower: lower,
          includeLower: includeLower,
          upper: upper,
          includeUpper: includeUpper,
        ),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  hdrModeEqualTo(String value, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.equalTo(
          property: r'hdrMode',
          value: value,
          caseSensitive: caseSensitive,
        ),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  hdrModeGreaterThan(
    String value, {
    bool include = false,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.greaterThan(
          include: include,
          property: r'hdrMode',
          value: value,
          caseSensitive: caseSensitive,
        ),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  hdrModeLessThan(
    String value, {
    bool include = false,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.lessThan(
          include: include,
          property: r'hdrMode',
          value: value,
          caseSensitive: caseSensitive,
        ),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  hdrModeBetween(
    String lower,
    String upper, {
    bool includeLower = true,
    bool includeUpper = true,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.between(
          property: r'hdrMode',
          lower: lower,
          includeLower: includeLower,
          upper: upper,
          includeUpper: includeUpper,
          caseSensitive: caseSensitive,
        ),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  hdrModeStartsWith(String value, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.startsWith(
          property: r'hdrMode',
          value: value,
          caseSensitive: caseSensitive,
        ),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  hdrModeEndsWith(String value, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.endsWith(
          property: r'hdrMode',
          value: value,
          caseSensitive: caseSensitive,
        ),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  hdrModeContains(String value, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.contains(
          property: r'hdrMode',
          value: value,
          caseSensitive: caseSensitive,
        ),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  hdrModeMatches(String pattern, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.matches(
          property: r'hdrMode',
          wildcard: pattern,
          caseSensitive: caseSensitive,
        ),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  hdrModeIsEmpty() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.equalTo(property: r'hdrMode', value: ''),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  hdrModeIsNotEmpty() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.greaterThan(property: r'hdrMode', value: ''),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  hwdecEqualTo(String value, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.equalTo(
          property: r'hwdec',
          value: value,
          caseSensitive: caseSensitive,
        ),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  hwdecGreaterThan(
    String value, {
    bool include = false,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.greaterThan(
          include: include,
          property: r'hwdec',
          value: value,
          caseSensitive: caseSensitive,
        ),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  hwdecLessThan(
    String value, {
    bool include = false,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.lessThan(
          include: include,
          property: r'hwdec',
          value: value,
          caseSensitive: caseSensitive,
        ),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  hwdecBetween(
    String lower,
    String upper, {
    bool includeLower = true,
    bool includeUpper = true,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.between(
          property: r'hwdec',
          lower: lower,
          includeLower: includeLower,
          upper: upper,
          includeUpper: includeUpper,
          caseSensitive: caseSensitive,
        ),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  hwdecStartsWith(String value, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.startsWith(
          property: r'hwdec',
          value: value,
          caseSensitive: caseSensitive,
        ),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  hwdecEndsWith(String value, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.endsWith(
          property: r'hwdec',
          value: value,
          caseSensitive: caseSensitive,
        ),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  hwdecContains(String value, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.contains(
          property: r'hwdec',
          value: value,
          caseSensitive: caseSensitive,
        ),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  hwdecMatches(String pattern, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.matches(
          property: r'hwdec',
          wildcard: pattern,
          caseSensitive: caseSensitive,
        ),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  hwdecIsEmpty() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.equalTo(property: r'hwdec', value: ''),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  hwdecIsNotEmpty() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.greaterThan(property: r'hwdec', value: ''),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  idEqualTo(Id value) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.equalTo(property: r'id', value: value),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  idGreaterThan(Id value, {bool include = false}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.greaterThan(
          include: include,
          property: r'id',
          value: value,
        ),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  idLessThan(Id value, {bool include = false}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.lessThan(
          include: include,
          property: r'id',
          value: value,
        ),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  idBetween(
    Id lower,
    Id upper, {
    bool includeLower = true,
    bool includeUpper = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.between(
          property: r'id',
          lower: lower,
          includeLower: includeLower,
          upper: upper,
          includeUpper: includeUpper,
        ),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  languageEqualTo(String value, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.equalTo(
          property: r'language',
          value: value,
          caseSensitive: caseSensitive,
        ),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  languageGreaterThan(
    String value, {
    bool include = false,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.greaterThan(
          include: include,
          property: r'language',
          value: value,
          caseSensitive: caseSensitive,
        ),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  languageLessThan(
    String value, {
    bool include = false,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.lessThan(
          include: include,
          property: r'language',
          value: value,
          caseSensitive: caseSensitive,
        ),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  languageBetween(
    String lower,
    String upper, {
    bool includeLower = true,
    bool includeUpper = true,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.between(
          property: r'language',
          lower: lower,
          includeLower: includeLower,
          upper: upper,
          includeUpper: includeUpper,
          caseSensitive: caseSensitive,
        ),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  languageStartsWith(String value, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.startsWith(
          property: r'language',
          value: value,
          caseSensitive: caseSensitive,
        ),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  languageEndsWith(String value, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.endsWith(
          property: r'language',
          value: value,
          caseSensitive: caseSensitive,
        ),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  languageContains(String value, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.contains(
          property: r'language',
          value: value,
          caseSensitive: caseSensitive,
        ),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  languageMatches(String pattern, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.matches(
          property: r'language',
          wildcard: pattern,
          caseSensitive: caseSensitive,
        ),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  languageIsEmpty() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.equalTo(property: r'language', value: ''),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  languageIsNotEmpty() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.greaterThan(property: r'language', value: ''),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  layoutEqualTo(String value, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.equalTo(
          property: r'layout',
          value: value,
          caseSensitive: caseSensitive,
        ),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  layoutGreaterThan(
    String value, {
    bool include = false,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.greaterThan(
          include: include,
          property: r'layout',
          value: value,
          caseSensitive: caseSensitive,
        ),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  layoutLessThan(
    String value, {
    bool include = false,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.lessThan(
          include: include,
          property: r'layout',
          value: value,
          caseSensitive: caseSensitive,
        ),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  layoutBetween(
    String lower,
    String upper, {
    bool includeLower = true,
    bool includeUpper = true,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.between(
          property: r'layout',
          lower: lower,
          includeLower: includeLower,
          upper: upper,
          includeUpper: includeUpper,
          caseSensitive: caseSensitive,
        ),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  layoutStartsWith(String value, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.startsWith(
          property: r'layout',
          value: value,
          caseSensitive: caseSensitive,
        ),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  layoutEndsWith(String value, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.endsWith(
          property: r'layout',
          value: value,
          caseSensitive: caseSensitive,
        ),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  layoutContains(String value, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.contains(
          property: r'layout',
          value: value,
          caseSensitive: caseSensitive,
        ),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  layoutMatches(String pattern, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.matches(
          property: r'layout',
          wildcard: pattern,
          caseSensitive: caseSensitive,
        ),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  layoutIsEmpty() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.equalTo(property: r'layout', value: ''),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  layoutIsNotEmpty() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.greaterThan(property: r'layout', value: ''),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  libraryFoldersElementEqualTo(String value, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.equalTo(
          property: r'libraryFolders',
          value: value,
          caseSensitive: caseSensitive,
        ),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  libraryFoldersElementGreaterThan(
    String value, {
    bool include = false,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.greaterThan(
          include: include,
          property: r'libraryFolders',
          value: value,
          caseSensitive: caseSensitive,
        ),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  libraryFoldersElementLessThan(
    String value, {
    bool include = false,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.lessThan(
          include: include,
          property: r'libraryFolders',
          value: value,
          caseSensitive: caseSensitive,
        ),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  libraryFoldersElementBetween(
    String lower,
    String upper, {
    bool includeLower = true,
    bool includeUpper = true,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.between(
          property: r'libraryFolders',
          lower: lower,
          includeLower: includeLower,
          upper: upper,
          includeUpper: includeUpper,
          caseSensitive: caseSensitive,
        ),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  libraryFoldersElementStartsWith(String value, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.startsWith(
          property: r'libraryFolders',
          value: value,
          caseSensitive: caseSensitive,
        ),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  libraryFoldersElementEndsWith(String value, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.endsWith(
          property: r'libraryFolders',
          value: value,
          caseSensitive: caseSensitive,
        ),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  libraryFoldersElementContains(String value, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.contains(
          property: r'libraryFolders',
          value: value,
          caseSensitive: caseSensitive,
        ),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  libraryFoldersElementMatches(String pattern, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.matches(
          property: r'libraryFolders',
          wildcard: pattern,
          caseSensitive: caseSensitive,
        ),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  libraryFoldersElementIsEmpty() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.equalTo(property: r'libraryFolders', value: ''),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  libraryFoldersElementIsNotEmpty() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.greaterThan(property: r'libraryFolders', value: ''),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  libraryFoldersLengthEqualTo(int length) {
    return QueryBuilder.apply(this, (query) {
      return query.listLength(r'libraryFolders', length, true, length, true);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  libraryFoldersIsEmpty() {
    return QueryBuilder.apply(this, (query) {
      return query.listLength(r'libraryFolders', 0, true, 0, true);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  libraryFoldersIsNotEmpty() {
    return QueryBuilder.apply(this, (query) {
      return query.listLength(r'libraryFolders', 0, false, 999999, true);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  libraryFoldersLengthLessThan(int length, {bool include = false}) {
    return QueryBuilder.apply(this, (query) {
      return query.listLength(r'libraryFolders', 0, true, length, include);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  libraryFoldersLengthGreaterThan(int length, {bool include = false}) {
    return QueryBuilder.apply(this, (query) {
      return query.listLength(r'libraryFolders', length, include, 999999, true);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  libraryFoldersLengthBetween(
    int lower,
    int upper, {
    bool includeLower = true,
    bool includeUpper = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.listLength(
        r'libraryFolders',
        lower,
        includeLower,
        upper,
        includeUpper,
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  loopModeEqualTo(bool value) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.equalTo(property: r'loopMode', value: value),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  minimizeToTrayEqualTo(bool value) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.equalTo(property: r'minimizeToTray', value: value),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  muteEqualTo(bool value) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.equalTo(property: r'mute', value: value),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  pauseOnBatteryEqualTo(bool value) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.equalTo(property: r'pauseOnBattery', value: value),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  pauseOnFullscreenEqualTo(bool value) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.equalTo(property: r'pauseOnFullscreen', value: value),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  playbackRateEqualTo(double value, {double epsilon = Query.epsilon}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.equalTo(
          property: r'playbackRate',
          value: value,

          epsilon: epsilon,
        ),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  playbackRateGreaterThan(
    double value, {
    bool include = false,
    double epsilon = Query.epsilon,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.greaterThan(
          include: include,
          property: r'playbackRate',
          value: value,

          epsilon: epsilon,
        ),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  playbackRateLessThan(
    double value, {
    bool include = false,
    double epsilon = Query.epsilon,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.lessThan(
          include: include,
          property: r'playbackRate',
          value: value,

          epsilon: epsilon,
        ),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  playbackRateBetween(
    double lower,
    double upper, {
    bool includeLower = true,
    bool includeUpper = true,
    double epsilon = Query.epsilon,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.between(
          property: r'playbackRate',
          lower: lower,
          includeLower: includeLower,
          upper: upper,
          includeUpper: includeUpper,

          epsilon: epsilon,
        ),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  preferredMonitorIsNull() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        const FilterCondition.isNull(property: r'preferredMonitor'),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  preferredMonitorIsNotNull() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        const FilterCondition.isNotNull(property: r'preferredMonitor'),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  preferredMonitorEqualTo(String? value, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.equalTo(
          property: r'preferredMonitor',
          value: value,
          caseSensitive: caseSensitive,
        ),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  preferredMonitorGreaterThan(
    String? value, {
    bool include = false,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.greaterThan(
          include: include,
          property: r'preferredMonitor',
          value: value,
          caseSensitive: caseSensitive,
        ),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  preferredMonitorLessThan(
    String? value, {
    bool include = false,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.lessThan(
          include: include,
          property: r'preferredMonitor',
          value: value,
          caseSensitive: caseSensitive,
        ),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  preferredMonitorBetween(
    String? lower,
    String? upper, {
    bool includeLower = true,
    bool includeUpper = true,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.between(
          property: r'preferredMonitor',
          lower: lower,
          includeLower: includeLower,
          upper: upper,
          includeUpper: includeUpper,
          caseSensitive: caseSensitive,
        ),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  preferredMonitorStartsWith(String value, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.startsWith(
          property: r'preferredMonitor',
          value: value,
          caseSensitive: caseSensitive,
        ),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  preferredMonitorEndsWith(String value, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.endsWith(
          property: r'preferredMonitor',
          value: value,
          caseSensitive: caseSensitive,
        ),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  preferredMonitorContains(String value, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.contains(
          property: r'preferredMonitor',
          value: value,
          caseSensitive: caseSensitive,
        ),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  preferredMonitorMatches(String pattern, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.matches(
          property: r'preferredMonitor',
          wildcard: pattern,
          caseSensitive: caseSensitive,
        ),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  preferredMonitorIsEmpty() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.equalTo(property: r'preferredMonitor', value: ''),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  preferredMonitorIsNotEmpty() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.greaterThan(property: r'preferredMonitor', value: ''),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  rendererEqualTo(String value, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.equalTo(
          property: r'renderer',
          value: value,
          caseSensitive: caseSensitive,
        ),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  rendererGreaterThan(
    String value, {
    bool include = false,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.greaterThan(
          include: include,
          property: r'renderer',
          value: value,
          caseSensitive: caseSensitive,
        ),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  rendererLessThan(
    String value, {
    bool include = false,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.lessThan(
          include: include,
          property: r'renderer',
          value: value,
          caseSensitive: caseSensitive,
        ),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  rendererBetween(
    String lower,
    String upper, {
    bool includeLower = true,
    bool includeUpper = true,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.between(
          property: r'renderer',
          lower: lower,
          includeLower: includeLower,
          upper: upper,
          includeUpper: includeUpper,
          caseSensitive: caseSensitive,
        ),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  rendererStartsWith(String value, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.startsWith(
          property: r'renderer',
          value: value,
          caseSensitive: caseSensitive,
        ),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  rendererEndsWith(String value, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.endsWith(
          property: r'renderer',
          value: value,
          caseSensitive: caseSensitive,
        ),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  rendererContains(String value, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.contains(
          property: r'renderer',
          value: value,
          caseSensitive: caseSensitive,
        ),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  rendererMatches(String pattern, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.matches(
          property: r'renderer',
          wildcard: pattern,
          caseSensitive: caseSensitive,
        ),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  rendererIsEmpty() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.equalTo(property: r'renderer', value: ''),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  rendererIsNotEmpty() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.greaterThan(property: r'renderer', value: ''),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  restoreLastWallpaperEqualTo(bool value) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.equalTo(
          property: r'restoreLastWallpaper',
          value: value,
        ),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  shuffleEqualTo(bool value) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.equalTo(property: r'shuffle', value: value),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  sidebarCollapsedEqualTo(bool value) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.equalTo(property: r'sidebarCollapsed', value: value),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  startMinimizedEqualTo(bool value) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.equalTo(property: r'startMinimized', value: value),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  startTimeEqualTo(double value, {double epsilon = Query.epsilon}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.equalTo(
          property: r'startTime',
          value: value,

          epsilon: epsilon,
        ),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  startTimeGreaterThan(
    double value, {
    bool include = false,
    double epsilon = Query.epsilon,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.greaterThan(
          include: include,
          property: r'startTime',
          value: value,

          epsilon: epsilon,
        ),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  startTimeLessThan(
    double value, {
    bool include = false,
    double epsilon = Query.epsilon,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.lessThan(
          include: include,
          property: r'startTime',
          value: value,

          epsilon: epsilon,
        ),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  startTimeBetween(
    double lower,
    double upper, {
    bool includeLower = true,
    bool includeUpper = true,
    double epsilon = Query.epsilon,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.between(
          property: r'startTime',
          lower: lower,
          includeLower: includeLower,
          upper: upper,
          includeUpper: includeUpper,

          epsilon: epsilon,
        ),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  themeEqualTo(String value, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.equalTo(
          property: r'theme',
          value: value,
          caseSensitive: caseSensitive,
        ),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  themeGreaterThan(
    String value, {
    bool include = false,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.greaterThan(
          include: include,
          property: r'theme',
          value: value,
          caseSensitive: caseSensitive,
        ),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  themeLessThan(
    String value, {
    bool include = false,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.lessThan(
          include: include,
          property: r'theme',
          value: value,
          caseSensitive: caseSensitive,
        ),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  themeBetween(
    String lower,
    String upper, {
    bool includeLower = true,
    bool includeUpper = true,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.between(
          property: r'theme',
          lower: lower,
          includeLower: includeLower,
          upper: upper,
          includeUpper: includeUpper,
          caseSensitive: caseSensitive,
        ),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  themeStartsWith(String value, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.startsWith(
          property: r'theme',
          value: value,
          caseSensitive: caseSensitive,
        ),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  themeEndsWith(String value, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.endsWith(
          property: r'theme',
          value: value,
          caseSensitive: caseSensitive,
        ),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  themeContains(String value, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.contains(
          property: r'theme',
          value: value,
          caseSensitive: caseSensitive,
        ),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  themeMatches(String pattern, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.matches(
          property: r'theme',
          wildcard: pattern,
          caseSensitive: caseSensitive,
        ),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  themeIsEmpty() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.equalTo(property: r'theme', value: ''),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  themeIsNotEmpty() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.greaterThan(property: r'theme', value: ''),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  themeColorEqualTo(int value) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.equalTo(property: r'themeColor', value: value),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  themeColorGreaterThan(int value, {bool include = false}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.greaterThan(
          include: include,
          property: r'themeColor',
          value: value,
        ),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  themeColorLessThan(int value, {bool include = false}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.lessThan(
          include: include,
          property: r'themeColor',
          value: value,
        ),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  themeColorBetween(
    int lower,
    int upper, {
    bool includeLower = true,
    bool includeUpper = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.between(
          property: r'themeColor',
          lower: lower,
          includeLower: includeLower,
          upper: upper,
          includeUpper: includeUpper,
        ),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  toneMappingAlgorithmEqualTo(String value, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.equalTo(
          property: r'toneMappingAlgorithm',
          value: value,
          caseSensitive: caseSensitive,
        ),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  toneMappingAlgorithmGreaterThan(
    String value, {
    bool include = false,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.greaterThan(
          include: include,
          property: r'toneMappingAlgorithm',
          value: value,
          caseSensitive: caseSensitive,
        ),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  toneMappingAlgorithmLessThan(
    String value, {
    bool include = false,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.lessThan(
          include: include,
          property: r'toneMappingAlgorithm',
          value: value,
          caseSensitive: caseSensitive,
        ),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  toneMappingAlgorithmBetween(
    String lower,
    String upper, {
    bool includeLower = true,
    bool includeUpper = true,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.between(
          property: r'toneMappingAlgorithm',
          lower: lower,
          includeLower: includeLower,
          upper: upper,
          includeUpper: includeUpper,
          caseSensitive: caseSensitive,
        ),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  toneMappingAlgorithmStartsWith(String value, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.startsWith(
          property: r'toneMappingAlgorithm',
          value: value,
          caseSensitive: caseSensitive,
        ),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  toneMappingAlgorithmEndsWith(String value, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.endsWith(
          property: r'toneMappingAlgorithm',
          value: value,
          caseSensitive: caseSensitive,
        ),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  toneMappingAlgorithmContains(String value, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.contains(
          property: r'toneMappingAlgorithm',
          value: value,
          caseSensitive: caseSensitive,
        ),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  toneMappingAlgorithmMatches(String pattern, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.matches(
          property: r'toneMappingAlgorithm',
          wildcard: pattern,
          caseSensitive: caseSensitive,
        ),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  toneMappingAlgorithmIsEmpty() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.equalTo(property: r'toneMappingAlgorithm', value: ''),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  toneMappingAlgorithmIsNotEmpty() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.greaterThan(
          property: r'toneMappingAlgorithm',
          value: '',
        ),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  toneMappingComputePeakEqualTo(bool value) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.equalTo(
          property: r'toneMappingComputePeak',
          value: value,
        ),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  toneMappingModeEqualTo(String value, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.equalTo(
          property: r'toneMappingMode',
          value: value,
          caseSensitive: caseSensitive,
        ),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  toneMappingModeGreaterThan(
    String value, {
    bool include = false,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.greaterThan(
          include: include,
          property: r'toneMappingMode',
          value: value,
          caseSensitive: caseSensitive,
        ),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  toneMappingModeLessThan(
    String value, {
    bool include = false,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.lessThan(
          include: include,
          property: r'toneMappingMode',
          value: value,
          caseSensitive: caseSensitive,
        ),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  toneMappingModeBetween(
    String lower,
    String upper, {
    bool includeLower = true,
    bool includeUpper = true,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.between(
          property: r'toneMappingMode',
          lower: lower,
          includeLower: includeLower,
          upper: upper,
          includeUpper: includeUpper,
          caseSensitive: caseSensitive,
        ),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  toneMappingModeStartsWith(String value, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.startsWith(
          property: r'toneMappingMode',
          value: value,
          caseSensitive: caseSensitive,
        ),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  toneMappingModeEndsWith(String value, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.endsWith(
          property: r'toneMappingMode',
          value: value,
          caseSensitive: caseSensitive,
        ),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  toneMappingModeContains(String value, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.contains(
          property: r'toneMappingMode',
          value: value,
          caseSensitive: caseSensitive,
        ),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  toneMappingModeMatches(String pattern, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.matches(
          property: r'toneMappingMode',
          wildcard: pattern,
          caseSensitive: caseSensitive,
        ),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  toneMappingModeIsEmpty() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.equalTo(property: r'toneMappingMode', value: ''),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  toneMappingModeIsNotEmpty() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.greaterThan(property: r'toneMappingMode', value: ''),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  toneMappingParamEqualTo(double value, {double epsilon = Query.epsilon}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.equalTo(
          property: r'toneMappingParam',
          value: value,

          epsilon: epsilon,
        ),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  toneMappingParamGreaterThan(
    double value, {
    bool include = false,
    double epsilon = Query.epsilon,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.greaterThan(
          include: include,
          property: r'toneMappingParam',
          value: value,

          epsilon: epsilon,
        ),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  toneMappingParamLessThan(
    double value, {
    bool include = false,
    double epsilon = Query.epsilon,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.lessThan(
          include: include,
          property: r'toneMappingParam',
          value: value,

          epsilon: epsilon,
        ),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  toneMappingParamBetween(
    double lower,
    double upper, {
    bool includeLower = true,
    bool includeUpper = true,
    double epsilon = Query.epsilon,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.between(
          property: r'toneMappingParam',
          lower: lower,
          includeLower: includeLower,
          upper: upper,
          includeUpper: includeUpper,

          epsilon: epsilon,
        ),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  volumeEqualTo(double value, {double epsilon = Query.epsilon}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.equalTo(
          property: r'volume',
          value: value,

          epsilon: epsilon,
        ),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  volumeGreaterThan(
    double value, {
    bool include = false,
    double epsilon = Query.epsilon,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.greaterThan(
          include: include,
          property: r'volume',
          value: value,

          epsilon: epsilon,
        ),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  volumeLessThan(
    double value, {
    bool include = false,
    double epsilon = Query.epsilon,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.lessThan(
          include: include,
          property: r'volume',
          value: value,

          epsilon: epsilon,
        ),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  volumeBetween(
    double lower,
    double upper, {
    bool includeLower = true,
    bool includeUpper = true,
    double epsilon = Query.epsilon,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.between(
          property: r'volume',
          lower: lower,
          includeLower: includeLower,
          upper: upper,
          includeUpper: includeUpper,

          epsilon: epsilon,
        ),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  windowHeightEqualTo(int value) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.equalTo(property: r'windowHeight', value: value),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  windowHeightGreaterThan(int value, {bool include = false}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.greaterThan(
          include: include,
          property: r'windowHeight',
          value: value,
        ),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  windowHeightLessThan(int value, {bool include = false}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.lessThan(
          include: include,
          property: r'windowHeight',
          value: value,
        ),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  windowHeightBetween(
    int lower,
    int upper, {
    bool includeLower = true,
    bool includeUpper = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.between(
          property: r'windowHeight',
          lower: lower,
          includeLower: includeLower,
          upper: upper,
          includeUpper: includeUpper,
        ),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  windowWidthEqualTo(int value) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.equalTo(property: r'windowWidth', value: value),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  windowWidthGreaterThan(int value, {bool include = false}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.greaterThan(
          include: include,
          property: r'windowWidth',
          value: value,
        ),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  windowWidthLessThan(int value, {bool include = false}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.lessThan(
          include: include,
          property: r'windowWidth',
          value: value,
        ),
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterFilterCondition>
  windowWidthBetween(
    int lower,
    int upper, {
    bool includeLower = true,
    bool includeUpper = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.between(
          property: r'windowWidth',
          lower: lower,
          includeLower: includeLower,
          upper: upper,
          includeUpper: includeUpper,
        ),
      );
    });
  }
}

extension AppSettingsRecordQueryObject
    on QueryBuilder<AppSettingsRecord, AppSettingsRecord, QFilterCondition> {}

extension AppSettingsRecordQueryLinks
    on QueryBuilder<AppSettingsRecord, AppSettingsRecord, QFilterCondition> {}

extension AppSettingsRecordQuerySortBy
    on QueryBuilder<AppSettingsRecord, AppSettingsRecord, QSortBy> {
  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterSortBy>
  sortByAutostartEnabled() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'autostartEnabled', Sort.asc);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterSortBy>
  sortByAutostartEnabledDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'autostartEnabled', Sort.desc);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterSortBy>
  sortByBatteryFpsLimit() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'batteryFpsLimit', Sort.asc);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterSortBy>
  sortByBatteryFpsLimitDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'batteryFpsLimit', Sort.desc);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterSortBy>
  sortByColorSchemeVariant() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'colorSchemeVariant', Sort.asc);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterSortBy>
  sortByColorSchemeVariantDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'colorSchemeVariant', Sort.desc);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterSortBy>
  sortByDetailPanelVisible() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'detailPanelVisible', Sort.asc);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterSortBy>
  sortByDetailPanelVisibleDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'detailPanelVisible', Sort.desc);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterSortBy>
  sortByFpsLimit() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'fpsLimit', Sort.asc);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterSortBy>
  sortByFpsLimitDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'fpsLimit', Sort.desc);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterSortBy>
  sortByHdrMode() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'hdrMode', Sort.asc);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterSortBy>
  sortByHdrModeDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'hdrMode', Sort.desc);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterSortBy>
  sortByHwdec() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'hwdec', Sort.asc);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterSortBy>
  sortByHwdecDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'hwdec', Sort.desc);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterSortBy>
  sortByLanguage() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'language', Sort.asc);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterSortBy>
  sortByLanguageDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'language', Sort.desc);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterSortBy>
  sortByLayout() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'layout', Sort.asc);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterSortBy>
  sortByLayoutDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'layout', Sort.desc);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterSortBy>
  sortByLoopMode() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'loopMode', Sort.asc);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterSortBy>
  sortByLoopModeDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'loopMode', Sort.desc);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterSortBy>
  sortByMinimizeToTray() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'minimizeToTray', Sort.asc);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterSortBy>
  sortByMinimizeToTrayDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'minimizeToTray', Sort.desc);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterSortBy>
  sortByMute() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'mute', Sort.asc);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterSortBy>
  sortByMuteDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'mute', Sort.desc);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterSortBy>
  sortByPauseOnBattery() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'pauseOnBattery', Sort.asc);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterSortBy>
  sortByPauseOnBatteryDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'pauseOnBattery', Sort.desc);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterSortBy>
  sortByPauseOnFullscreen() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'pauseOnFullscreen', Sort.asc);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterSortBy>
  sortByPauseOnFullscreenDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'pauseOnFullscreen', Sort.desc);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterSortBy>
  sortByPlaybackRate() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'playbackRate', Sort.asc);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterSortBy>
  sortByPlaybackRateDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'playbackRate', Sort.desc);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterSortBy>
  sortByPreferredMonitor() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'preferredMonitor', Sort.asc);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterSortBy>
  sortByPreferredMonitorDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'preferredMonitor', Sort.desc);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterSortBy>
  sortByRenderer() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'renderer', Sort.asc);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterSortBy>
  sortByRendererDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'renderer', Sort.desc);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterSortBy>
  sortByRestoreLastWallpaper() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'restoreLastWallpaper', Sort.asc);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterSortBy>
  sortByRestoreLastWallpaperDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'restoreLastWallpaper', Sort.desc);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterSortBy>
  sortByShuffle() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'shuffle', Sort.asc);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterSortBy>
  sortByShuffleDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'shuffle', Sort.desc);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterSortBy>
  sortBySidebarCollapsed() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'sidebarCollapsed', Sort.asc);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterSortBy>
  sortBySidebarCollapsedDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'sidebarCollapsed', Sort.desc);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterSortBy>
  sortByStartMinimized() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'startMinimized', Sort.asc);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterSortBy>
  sortByStartMinimizedDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'startMinimized', Sort.desc);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterSortBy>
  sortByStartTime() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'startTime', Sort.asc);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterSortBy>
  sortByStartTimeDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'startTime', Sort.desc);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterSortBy>
  sortByTheme() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'theme', Sort.asc);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterSortBy>
  sortByThemeDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'theme', Sort.desc);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterSortBy>
  sortByThemeColor() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'themeColor', Sort.asc);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterSortBy>
  sortByThemeColorDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'themeColor', Sort.desc);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterSortBy>
  sortByToneMappingAlgorithm() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'toneMappingAlgorithm', Sort.asc);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterSortBy>
  sortByToneMappingAlgorithmDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'toneMappingAlgorithm', Sort.desc);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterSortBy>
  sortByToneMappingComputePeak() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'toneMappingComputePeak', Sort.asc);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterSortBy>
  sortByToneMappingComputePeakDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'toneMappingComputePeak', Sort.desc);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterSortBy>
  sortByToneMappingMode() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'toneMappingMode', Sort.asc);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterSortBy>
  sortByToneMappingModeDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'toneMappingMode', Sort.desc);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterSortBy>
  sortByToneMappingParam() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'toneMappingParam', Sort.asc);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterSortBy>
  sortByToneMappingParamDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'toneMappingParam', Sort.desc);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterSortBy>
  sortByVolume() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'volume', Sort.asc);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterSortBy>
  sortByVolumeDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'volume', Sort.desc);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterSortBy>
  sortByWindowHeight() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'windowHeight', Sort.asc);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterSortBy>
  sortByWindowHeightDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'windowHeight', Sort.desc);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterSortBy>
  sortByWindowWidth() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'windowWidth', Sort.asc);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterSortBy>
  sortByWindowWidthDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'windowWidth', Sort.desc);
    });
  }
}

extension AppSettingsRecordQuerySortThenBy
    on QueryBuilder<AppSettingsRecord, AppSettingsRecord, QSortThenBy> {
  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterSortBy>
  thenByAutostartEnabled() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'autostartEnabled', Sort.asc);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterSortBy>
  thenByAutostartEnabledDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'autostartEnabled', Sort.desc);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterSortBy>
  thenByBatteryFpsLimit() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'batteryFpsLimit', Sort.asc);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterSortBy>
  thenByBatteryFpsLimitDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'batteryFpsLimit', Sort.desc);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterSortBy>
  thenByColorSchemeVariant() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'colorSchemeVariant', Sort.asc);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterSortBy>
  thenByColorSchemeVariantDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'colorSchemeVariant', Sort.desc);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterSortBy>
  thenByDetailPanelVisible() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'detailPanelVisible', Sort.asc);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterSortBy>
  thenByDetailPanelVisibleDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'detailPanelVisible', Sort.desc);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterSortBy>
  thenByFpsLimit() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'fpsLimit', Sort.asc);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterSortBy>
  thenByFpsLimitDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'fpsLimit', Sort.desc);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterSortBy>
  thenByHdrMode() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'hdrMode', Sort.asc);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterSortBy>
  thenByHdrModeDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'hdrMode', Sort.desc);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterSortBy>
  thenByHwdec() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'hwdec', Sort.asc);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterSortBy>
  thenByHwdecDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'hwdec', Sort.desc);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterSortBy> thenById() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'id', Sort.asc);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterSortBy>
  thenByIdDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'id', Sort.desc);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterSortBy>
  thenByLanguage() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'language', Sort.asc);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterSortBy>
  thenByLanguageDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'language', Sort.desc);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterSortBy>
  thenByLayout() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'layout', Sort.asc);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterSortBy>
  thenByLayoutDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'layout', Sort.desc);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterSortBy>
  thenByLoopMode() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'loopMode', Sort.asc);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterSortBy>
  thenByLoopModeDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'loopMode', Sort.desc);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterSortBy>
  thenByMinimizeToTray() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'minimizeToTray', Sort.asc);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterSortBy>
  thenByMinimizeToTrayDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'minimizeToTray', Sort.desc);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterSortBy>
  thenByMute() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'mute', Sort.asc);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterSortBy>
  thenByMuteDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'mute', Sort.desc);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterSortBy>
  thenByPauseOnBattery() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'pauseOnBattery', Sort.asc);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterSortBy>
  thenByPauseOnBatteryDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'pauseOnBattery', Sort.desc);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterSortBy>
  thenByPauseOnFullscreen() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'pauseOnFullscreen', Sort.asc);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterSortBy>
  thenByPauseOnFullscreenDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'pauseOnFullscreen', Sort.desc);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterSortBy>
  thenByPlaybackRate() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'playbackRate', Sort.asc);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterSortBy>
  thenByPlaybackRateDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'playbackRate', Sort.desc);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterSortBy>
  thenByPreferredMonitor() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'preferredMonitor', Sort.asc);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterSortBy>
  thenByPreferredMonitorDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'preferredMonitor', Sort.desc);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterSortBy>
  thenByRenderer() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'renderer', Sort.asc);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterSortBy>
  thenByRendererDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'renderer', Sort.desc);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterSortBy>
  thenByRestoreLastWallpaper() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'restoreLastWallpaper', Sort.asc);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterSortBy>
  thenByRestoreLastWallpaperDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'restoreLastWallpaper', Sort.desc);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterSortBy>
  thenByShuffle() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'shuffle', Sort.asc);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterSortBy>
  thenByShuffleDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'shuffle', Sort.desc);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterSortBy>
  thenBySidebarCollapsed() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'sidebarCollapsed', Sort.asc);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterSortBy>
  thenBySidebarCollapsedDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'sidebarCollapsed', Sort.desc);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterSortBy>
  thenByStartMinimized() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'startMinimized', Sort.asc);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterSortBy>
  thenByStartMinimizedDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'startMinimized', Sort.desc);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterSortBy>
  thenByStartTime() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'startTime', Sort.asc);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterSortBy>
  thenByStartTimeDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'startTime', Sort.desc);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterSortBy>
  thenByTheme() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'theme', Sort.asc);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterSortBy>
  thenByThemeDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'theme', Sort.desc);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterSortBy>
  thenByThemeColor() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'themeColor', Sort.asc);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterSortBy>
  thenByThemeColorDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'themeColor', Sort.desc);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterSortBy>
  thenByToneMappingAlgorithm() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'toneMappingAlgorithm', Sort.asc);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterSortBy>
  thenByToneMappingAlgorithmDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'toneMappingAlgorithm', Sort.desc);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterSortBy>
  thenByToneMappingComputePeak() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'toneMappingComputePeak', Sort.asc);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterSortBy>
  thenByToneMappingComputePeakDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'toneMappingComputePeak', Sort.desc);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterSortBy>
  thenByToneMappingMode() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'toneMappingMode', Sort.asc);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterSortBy>
  thenByToneMappingModeDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'toneMappingMode', Sort.desc);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterSortBy>
  thenByToneMappingParam() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'toneMappingParam', Sort.asc);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterSortBy>
  thenByToneMappingParamDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'toneMappingParam', Sort.desc);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterSortBy>
  thenByVolume() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'volume', Sort.asc);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterSortBy>
  thenByVolumeDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'volume', Sort.desc);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterSortBy>
  thenByWindowHeight() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'windowHeight', Sort.asc);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterSortBy>
  thenByWindowHeightDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'windowHeight', Sort.desc);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterSortBy>
  thenByWindowWidth() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'windowWidth', Sort.asc);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QAfterSortBy>
  thenByWindowWidthDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'windowWidth', Sort.desc);
    });
  }
}

extension AppSettingsRecordQueryWhereDistinct
    on QueryBuilder<AppSettingsRecord, AppSettingsRecord, QDistinct> {
  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QDistinct>
  distinctByAutostartEnabled() {
    return QueryBuilder.apply(this, (query) {
      return query.addDistinctBy(r'autostartEnabled');
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QDistinct>
  distinctByBatteryFpsLimit() {
    return QueryBuilder.apply(this, (query) {
      return query.addDistinctBy(r'batteryFpsLimit');
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QDistinct>
  distinctByColorSchemeVariant({bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addDistinctBy(
        r'colorSchemeVariant',
        caseSensitive: caseSensitive,
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QDistinct>
  distinctByCustomThemeColors() {
    return QueryBuilder.apply(this, (query) {
      return query.addDistinctBy(r'customThemeColors');
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QDistinct>
  distinctByDetailPanelVisible() {
    return QueryBuilder.apply(this, (query) {
      return query.addDistinctBy(r'detailPanelVisible');
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QDistinct>
  distinctByFpsLimit() {
    return QueryBuilder.apply(this, (query) {
      return query.addDistinctBy(r'fpsLimit');
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QDistinct>
  distinctByHdrMode({bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addDistinctBy(r'hdrMode', caseSensitive: caseSensitive);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QDistinct>
  distinctByHwdec({bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addDistinctBy(r'hwdec', caseSensitive: caseSensitive);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QDistinct>
  distinctByLanguage({bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addDistinctBy(r'language', caseSensitive: caseSensitive);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QDistinct>
  distinctByLayout({bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addDistinctBy(r'layout', caseSensitive: caseSensitive);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QDistinct>
  distinctByLibraryFolders() {
    return QueryBuilder.apply(this, (query) {
      return query.addDistinctBy(r'libraryFolders');
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QDistinct>
  distinctByLoopMode() {
    return QueryBuilder.apply(this, (query) {
      return query.addDistinctBy(r'loopMode');
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QDistinct>
  distinctByMinimizeToTray() {
    return QueryBuilder.apply(this, (query) {
      return query.addDistinctBy(r'minimizeToTray');
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QDistinct>
  distinctByMute() {
    return QueryBuilder.apply(this, (query) {
      return query.addDistinctBy(r'mute');
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QDistinct>
  distinctByPauseOnBattery() {
    return QueryBuilder.apply(this, (query) {
      return query.addDistinctBy(r'pauseOnBattery');
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QDistinct>
  distinctByPauseOnFullscreen() {
    return QueryBuilder.apply(this, (query) {
      return query.addDistinctBy(r'pauseOnFullscreen');
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QDistinct>
  distinctByPlaybackRate() {
    return QueryBuilder.apply(this, (query) {
      return query.addDistinctBy(r'playbackRate');
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QDistinct>
  distinctByPreferredMonitor({bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addDistinctBy(
        r'preferredMonitor',
        caseSensitive: caseSensitive,
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QDistinct>
  distinctByRenderer({bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addDistinctBy(r'renderer', caseSensitive: caseSensitive);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QDistinct>
  distinctByRestoreLastWallpaper() {
    return QueryBuilder.apply(this, (query) {
      return query.addDistinctBy(r'restoreLastWallpaper');
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QDistinct>
  distinctByShuffle() {
    return QueryBuilder.apply(this, (query) {
      return query.addDistinctBy(r'shuffle');
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QDistinct>
  distinctBySidebarCollapsed() {
    return QueryBuilder.apply(this, (query) {
      return query.addDistinctBy(r'sidebarCollapsed');
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QDistinct>
  distinctByStartMinimized() {
    return QueryBuilder.apply(this, (query) {
      return query.addDistinctBy(r'startMinimized');
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QDistinct>
  distinctByStartTime() {
    return QueryBuilder.apply(this, (query) {
      return query.addDistinctBy(r'startTime');
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QDistinct>
  distinctByTheme({bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addDistinctBy(r'theme', caseSensitive: caseSensitive);
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QDistinct>
  distinctByThemeColor() {
    return QueryBuilder.apply(this, (query) {
      return query.addDistinctBy(r'themeColor');
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QDistinct>
  distinctByToneMappingAlgorithm({bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addDistinctBy(
        r'toneMappingAlgorithm',
        caseSensitive: caseSensitive,
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QDistinct>
  distinctByToneMappingComputePeak() {
    return QueryBuilder.apply(this, (query) {
      return query.addDistinctBy(r'toneMappingComputePeak');
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QDistinct>
  distinctByToneMappingMode({bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addDistinctBy(
        r'toneMappingMode',
        caseSensitive: caseSensitive,
      );
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QDistinct>
  distinctByToneMappingParam() {
    return QueryBuilder.apply(this, (query) {
      return query.addDistinctBy(r'toneMappingParam');
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QDistinct>
  distinctByVolume() {
    return QueryBuilder.apply(this, (query) {
      return query.addDistinctBy(r'volume');
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QDistinct>
  distinctByWindowHeight() {
    return QueryBuilder.apply(this, (query) {
      return query.addDistinctBy(r'windowHeight');
    });
  }

  QueryBuilder<AppSettingsRecord, AppSettingsRecord, QDistinct>
  distinctByWindowWidth() {
    return QueryBuilder.apply(this, (query) {
      return query.addDistinctBy(r'windowWidth');
    });
  }
}

extension AppSettingsRecordQueryProperty
    on QueryBuilder<AppSettingsRecord, AppSettingsRecord, QQueryProperty> {
  QueryBuilder<AppSettingsRecord, int, QQueryOperations> idProperty() {
    return QueryBuilder.apply(this, (query) {
      return query.addPropertyName(r'id');
    });
  }

  QueryBuilder<AppSettingsRecord, bool, QQueryOperations>
  autostartEnabledProperty() {
    return QueryBuilder.apply(this, (query) {
      return query.addPropertyName(r'autostartEnabled');
    });
  }

  QueryBuilder<AppSettingsRecord, int?, QQueryOperations>
  batteryFpsLimitProperty() {
    return QueryBuilder.apply(this, (query) {
      return query.addPropertyName(r'batteryFpsLimit');
    });
  }

  QueryBuilder<AppSettingsRecord, String, QQueryOperations>
  colorSchemeVariantProperty() {
    return QueryBuilder.apply(this, (query) {
      return query.addPropertyName(r'colorSchemeVariant');
    });
  }

  QueryBuilder<AppSettingsRecord, List<int>, QQueryOperations>
  customThemeColorsProperty() {
    return QueryBuilder.apply(this, (query) {
      return query.addPropertyName(r'customThemeColors');
    });
  }

  QueryBuilder<AppSettingsRecord, bool, QQueryOperations>
  detailPanelVisibleProperty() {
    return QueryBuilder.apply(this, (query) {
      return query.addPropertyName(r'detailPanelVisible');
    });
  }

  QueryBuilder<AppSettingsRecord, int?, QQueryOperations> fpsLimitProperty() {
    return QueryBuilder.apply(this, (query) {
      return query.addPropertyName(r'fpsLimit');
    });
  }

  QueryBuilder<AppSettingsRecord, String, QQueryOperations> hdrModeProperty() {
    return QueryBuilder.apply(this, (query) {
      return query.addPropertyName(r'hdrMode');
    });
  }

  QueryBuilder<AppSettingsRecord, String, QQueryOperations> hwdecProperty() {
    return QueryBuilder.apply(this, (query) {
      return query.addPropertyName(r'hwdec');
    });
  }

  QueryBuilder<AppSettingsRecord, String, QQueryOperations> languageProperty() {
    return QueryBuilder.apply(this, (query) {
      return query.addPropertyName(r'language');
    });
  }

  QueryBuilder<AppSettingsRecord, String, QQueryOperations> layoutProperty() {
    return QueryBuilder.apply(this, (query) {
      return query.addPropertyName(r'layout');
    });
  }

  QueryBuilder<AppSettingsRecord, List<String>, QQueryOperations>
  libraryFoldersProperty() {
    return QueryBuilder.apply(this, (query) {
      return query.addPropertyName(r'libraryFolders');
    });
  }

  QueryBuilder<AppSettingsRecord, bool, QQueryOperations> loopModeProperty() {
    return QueryBuilder.apply(this, (query) {
      return query.addPropertyName(r'loopMode');
    });
  }

  QueryBuilder<AppSettingsRecord, bool, QQueryOperations>
  minimizeToTrayProperty() {
    return QueryBuilder.apply(this, (query) {
      return query.addPropertyName(r'minimizeToTray');
    });
  }

  QueryBuilder<AppSettingsRecord, bool, QQueryOperations> muteProperty() {
    return QueryBuilder.apply(this, (query) {
      return query.addPropertyName(r'mute');
    });
  }

  QueryBuilder<AppSettingsRecord, bool, QQueryOperations>
  pauseOnBatteryProperty() {
    return QueryBuilder.apply(this, (query) {
      return query.addPropertyName(r'pauseOnBattery');
    });
  }

  QueryBuilder<AppSettingsRecord, bool, QQueryOperations>
  pauseOnFullscreenProperty() {
    return QueryBuilder.apply(this, (query) {
      return query.addPropertyName(r'pauseOnFullscreen');
    });
  }

  QueryBuilder<AppSettingsRecord, double, QQueryOperations>
  playbackRateProperty() {
    return QueryBuilder.apply(this, (query) {
      return query.addPropertyName(r'playbackRate');
    });
  }

  QueryBuilder<AppSettingsRecord, String?, QQueryOperations>
  preferredMonitorProperty() {
    return QueryBuilder.apply(this, (query) {
      return query.addPropertyName(r'preferredMonitor');
    });
  }

  QueryBuilder<AppSettingsRecord, String, QQueryOperations> rendererProperty() {
    return QueryBuilder.apply(this, (query) {
      return query.addPropertyName(r'renderer');
    });
  }

  QueryBuilder<AppSettingsRecord, bool, QQueryOperations>
  restoreLastWallpaperProperty() {
    return QueryBuilder.apply(this, (query) {
      return query.addPropertyName(r'restoreLastWallpaper');
    });
  }

  QueryBuilder<AppSettingsRecord, bool, QQueryOperations> shuffleProperty() {
    return QueryBuilder.apply(this, (query) {
      return query.addPropertyName(r'shuffle');
    });
  }

  QueryBuilder<AppSettingsRecord, bool, QQueryOperations>
  sidebarCollapsedProperty() {
    return QueryBuilder.apply(this, (query) {
      return query.addPropertyName(r'sidebarCollapsed');
    });
  }

  QueryBuilder<AppSettingsRecord, bool, QQueryOperations>
  startMinimizedProperty() {
    return QueryBuilder.apply(this, (query) {
      return query.addPropertyName(r'startMinimized');
    });
  }

  QueryBuilder<AppSettingsRecord, double, QQueryOperations>
  startTimeProperty() {
    return QueryBuilder.apply(this, (query) {
      return query.addPropertyName(r'startTime');
    });
  }

  QueryBuilder<AppSettingsRecord, String, QQueryOperations> themeProperty() {
    return QueryBuilder.apply(this, (query) {
      return query.addPropertyName(r'theme');
    });
  }

  QueryBuilder<AppSettingsRecord, int, QQueryOperations> themeColorProperty() {
    return QueryBuilder.apply(this, (query) {
      return query.addPropertyName(r'themeColor');
    });
  }

  QueryBuilder<AppSettingsRecord, String, QQueryOperations>
  toneMappingAlgorithmProperty() {
    return QueryBuilder.apply(this, (query) {
      return query.addPropertyName(r'toneMappingAlgorithm');
    });
  }

  QueryBuilder<AppSettingsRecord, bool, QQueryOperations>
  toneMappingComputePeakProperty() {
    return QueryBuilder.apply(this, (query) {
      return query.addPropertyName(r'toneMappingComputePeak');
    });
  }

  QueryBuilder<AppSettingsRecord, String, QQueryOperations>
  toneMappingModeProperty() {
    return QueryBuilder.apply(this, (query) {
      return query.addPropertyName(r'toneMappingMode');
    });
  }

  QueryBuilder<AppSettingsRecord, double, QQueryOperations>
  toneMappingParamProperty() {
    return QueryBuilder.apply(this, (query) {
      return query.addPropertyName(r'toneMappingParam');
    });
  }

  QueryBuilder<AppSettingsRecord, double, QQueryOperations> volumeProperty() {
    return QueryBuilder.apply(this, (query) {
      return query.addPropertyName(r'volume');
    });
  }

  QueryBuilder<AppSettingsRecord, int, QQueryOperations>
  windowHeightProperty() {
    return QueryBuilder.apply(this, (query) {
      return query.addPropertyName(r'windowHeight');
    });
  }

  QueryBuilder<AppSettingsRecord, int, QQueryOperations> windowWidthProperty() {
    return QueryBuilder.apply(this, (query) {
      return query.addPropertyName(r'windowWidth');
    });
  }
}

// coverage:ignore-file
// ignore_for_file: duplicate_ignore, non_constant_identifier_names, constant_identifier_names, invalid_use_of_protected_member, unnecessary_cast, prefer_const_constructors, lines_longer_than_80_chars, require_trailing_commas, inference_failure_on_function_invocation, unnecessary_parenthesis, unnecessary_raw_strings, unnecessary_null_checks, join_return_with_assignment, prefer_final_locals, avoid_js_rounded_ints, avoid_positional_boolean_parameters, always_specify_types

extension GetWallpaperAssignmentRecordCollection on Isar {
  IsarCollection<WallpaperAssignmentRecord> get wallpaperAssignmentRecords =>
      this.collection();
}

const WallpaperAssignmentRecordSchema = CollectionSchema(
  name: r'WallpaperAssignmentRecord',
  id: -6324685307849166934,
  properties: {
    r'output': PropertySchema(id: 0, name: r'output', type: IsarType.string),
    r'sourceId': PropertySchema(
      id: 1,
      name: r'sourceId',
      type: IsarType.string,
    ),
    r'sourcePath': PropertySchema(
      id: 2,
      name: r'sourcePath',
      type: IsarType.string,
    ),
  },

  estimateSize: _wallpaperAssignmentRecordEstimateSize,
  serialize: _wallpaperAssignmentRecordSerialize,
  deserialize: _wallpaperAssignmentRecordDeserialize,
  deserializeProp: _wallpaperAssignmentRecordDeserializeProp,
  idName: r'id',
  indexes: {
    r'output': IndexSchema(
      id: -8436553305763462561,
      name: r'output',
      unique: true,
      replace: false,
      properties: [
        IndexPropertySchema(
          name: r'output',
          type: IndexType.hash,
          caseSensitive: true,
        ),
      ],
    ),
  },
  links: {},
  embeddedSchemas: {},

  getId: _wallpaperAssignmentRecordGetId,
  getLinks: _wallpaperAssignmentRecordGetLinks,
  attach: _wallpaperAssignmentRecordAttach,
  version: '3.3.2',
);

int _wallpaperAssignmentRecordEstimateSize(
  WallpaperAssignmentRecord object,
  List<int> offsets,
  Map<Type, List<int>> allOffsets,
) {
  var bytesCount = offsets.last;
  bytesCount += 3 + object.output.length * 3;
  {
    final value = object.sourceId;
    if (value != null) {
      bytesCount += 3 + value.length * 3;
    }
  }
  bytesCount += 3 + object.sourcePath.length * 3;
  return bytesCount;
}

void _wallpaperAssignmentRecordSerialize(
  WallpaperAssignmentRecord object,
  IsarWriter writer,
  List<int> offsets,
  Map<Type, List<int>> allOffsets,
) {
  writer.writeString(offsets[0], object.output);
  writer.writeString(offsets[1], object.sourceId);
  writer.writeString(offsets[2], object.sourcePath);
}

WallpaperAssignmentRecord _wallpaperAssignmentRecordDeserialize(
  Id id,
  IsarReader reader,
  List<int> offsets,
  Map<Type, List<int>> allOffsets,
) {
  final object = WallpaperAssignmentRecord();
  object.id = id;
  object.output = reader.readString(offsets[0]);
  object.sourceId = reader.readStringOrNull(offsets[1]);
  object.sourcePath = reader.readString(offsets[2]);
  return object;
}

P _wallpaperAssignmentRecordDeserializeProp<P>(
  IsarReader reader,
  int propertyId,
  int offset,
  Map<Type, List<int>> allOffsets,
) {
  switch (propertyId) {
    case 0:
      return (reader.readString(offset)) as P;
    case 1:
      return (reader.readStringOrNull(offset)) as P;
    case 2:
      return (reader.readString(offset)) as P;
    default:
      throw IsarError('Unknown property with id $propertyId');
  }
}

Id _wallpaperAssignmentRecordGetId(WallpaperAssignmentRecord object) {
  return object.id;
}

List<IsarLinkBase<dynamic>> _wallpaperAssignmentRecordGetLinks(
  WallpaperAssignmentRecord object,
) {
  return [];
}

void _wallpaperAssignmentRecordAttach(
  IsarCollection<dynamic> col,
  Id id,
  WallpaperAssignmentRecord object,
) {
  object.id = id;
}

extension WallpaperAssignmentRecordByIndex
    on IsarCollection<WallpaperAssignmentRecord> {
  Future<WallpaperAssignmentRecord?> getByOutput(String output) {
    return getByIndex(r'output', [output]);
  }

  WallpaperAssignmentRecord? getByOutputSync(String output) {
    return getByIndexSync(r'output', [output]);
  }

  Future<bool> deleteByOutput(String output) {
    return deleteByIndex(r'output', [output]);
  }

  bool deleteByOutputSync(String output) {
    return deleteByIndexSync(r'output', [output]);
  }

  Future<List<WallpaperAssignmentRecord?>> getAllByOutput(
    List<String> outputValues,
  ) {
    final values = outputValues.map((e) => [e]).toList();
    return getAllByIndex(r'output', values);
  }

  List<WallpaperAssignmentRecord?> getAllByOutputSync(
    List<String> outputValues,
  ) {
    final values = outputValues.map((e) => [e]).toList();
    return getAllByIndexSync(r'output', values);
  }

  Future<int> deleteAllByOutput(List<String> outputValues) {
    final values = outputValues.map((e) => [e]).toList();
    return deleteAllByIndex(r'output', values);
  }

  int deleteAllByOutputSync(List<String> outputValues) {
    final values = outputValues.map((e) => [e]).toList();
    return deleteAllByIndexSync(r'output', values);
  }

  Future<Id> putByOutput(WallpaperAssignmentRecord object) {
    return putByIndex(r'output', object);
  }

  Id putByOutputSync(
    WallpaperAssignmentRecord object, {
    bool saveLinks = true,
  }) {
    return putByIndexSync(r'output', object, saveLinks: saveLinks);
  }

  Future<List<Id>> putAllByOutput(List<WallpaperAssignmentRecord> objects) {
    return putAllByIndex(r'output', objects);
  }

  List<Id> putAllByOutputSync(
    List<WallpaperAssignmentRecord> objects, {
    bool saveLinks = true,
  }) {
    return putAllByIndexSync(r'output', objects, saveLinks: saveLinks);
  }
}

extension WallpaperAssignmentRecordQueryWhereSort
    on
        QueryBuilder<
          WallpaperAssignmentRecord,
          WallpaperAssignmentRecord,
          QWhere
        > {
  QueryBuilder<
    WallpaperAssignmentRecord,
    WallpaperAssignmentRecord,
    QAfterWhere
  >
  anyId() {
    return QueryBuilder.apply(this, (query) {
      return query.addWhereClause(const IdWhereClause.any());
    });
  }
}

extension WallpaperAssignmentRecordQueryWhere
    on
        QueryBuilder<
          WallpaperAssignmentRecord,
          WallpaperAssignmentRecord,
          QWhereClause
        > {
  QueryBuilder<
    WallpaperAssignmentRecord,
    WallpaperAssignmentRecord,
    QAfterWhereClause
  >
  idEqualTo(Id id) {
    return QueryBuilder.apply(this, (query) {
      return query.addWhereClause(IdWhereClause.between(lower: id, upper: id));
    });
  }

  QueryBuilder<
    WallpaperAssignmentRecord,
    WallpaperAssignmentRecord,
    QAfterWhereClause
  >
  idNotEqualTo(Id id) {
    return QueryBuilder.apply(this, (query) {
      if (query.whereSort == Sort.asc) {
        return query
            .addWhereClause(
              IdWhereClause.lessThan(upper: id, includeUpper: false),
            )
            .addWhereClause(
              IdWhereClause.greaterThan(lower: id, includeLower: false),
            );
      } else {
        return query
            .addWhereClause(
              IdWhereClause.greaterThan(lower: id, includeLower: false),
            )
            .addWhereClause(
              IdWhereClause.lessThan(upper: id, includeUpper: false),
            );
      }
    });
  }

  QueryBuilder<
    WallpaperAssignmentRecord,
    WallpaperAssignmentRecord,
    QAfterWhereClause
  >
  idGreaterThan(Id id, {bool include = false}) {
    return QueryBuilder.apply(this, (query) {
      return query.addWhereClause(
        IdWhereClause.greaterThan(lower: id, includeLower: include),
      );
    });
  }

  QueryBuilder<
    WallpaperAssignmentRecord,
    WallpaperAssignmentRecord,
    QAfterWhereClause
  >
  idLessThan(Id id, {bool include = false}) {
    return QueryBuilder.apply(this, (query) {
      return query.addWhereClause(
        IdWhereClause.lessThan(upper: id, includeUpper: include),
      );
    });
  }

  QueryBuilder<
    WallpaperAssignmentRecord,
    WallpaperAssignmentRecord,
    QAfterWhereClause
  >
  idBetween(
    Id lowerId,
    Id upperId, {
    bool includeLower = true,
    bool includeUpper = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addWhereClause(
        IdWhereClause.between(
          lower: lowerId,
          includeLower: includeLower,
          upper: upperId,
          includeUpper: includeUpper,
        ),
      );
    });
  }

  QueryBuilder<
    WallpaperAssignmentRecord,
    WallpaperAssignmentRecord,
    QAfterWhereClause
  >
  outputEqualTo(String output) {
    return QueryBuilder.apply(this, (query) {
      return query.addWhereClause(
        IndexWhereClause.equalTo(indexName: r'output', value: [output]),
      );
    });
  }

  QueryBuilder<
    WallpaperAssignmentRecord,
    WallpaperAssignmentRecord,
    QAfterWhereClause
  >
  outputNotEqualTo(String output) {
    return QueryBuilder.apply(this, (query) {
      if (query.whereSort == Sort.asc) {
        return query
            .addWhereClause(
              IndexWhereClause.between(
                indexName: r'output',
                lower: [],
                upper: [output],
                includeUpper: false,
              ),
            )
            .addWhereClause(
              IndexWhereClause.between(
                indexName: r'output',
                lower: [output],
                includeLower: false,
                upper: [],
              ),
            );
      } else {
        return query
            .addWhereClause(
              IndexWhereClause.between(
                indexName: r'output',
                lower: [output],
                includeLower: false,
                upper: [],
              ),
            )
            .addWhereClause(
              IndexWhereClause.between(
                indexName: r'output',
                lower: [],
                upper: [output],
                includeUpper: false,
              ),
            );
      }
    });
  }
}

extension WallpaperAssignmentRecordQueryFilter
    on
        QueryBuilder<
          WallpaperAssignmentRecord,
          WallpaperAssignmentRecord,
          QFilterCondition
        > {
  QueryBuilder<
    WallpaperAssignmentRecord,
    WallpaperAssignmentRecord,
    QAfterFilterCondition
  >
  idEqualTo(Id value) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.equalTo(property: r'id', value: value),
      );
    });
  }

  QueryBuilder<
    WallpaperAssignmentRecord,
    WallpaperAssignmentRecord,
    QAfterFilterCondition
  >
  idGreaterThan(Id value, {bool include = false}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.greaterThan(
          include: include,
          property: r'id',
          value: value,
        ),
      );
    });
  }

  QueryBuilder<
    WallpaperAssignmentRecord,
    WallpaperAssignmentRecord,
    QAfterFilterCondition
  >
  idLessThan(Id value, {bool include = false}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.lessThan(
          include: include,
          property: r'id',
          value: value,
        ),
      );
    });
  }

  QueryBuilder<
    WallpaperAssignmentRecord,
    WallpaperAssignmentRecord,
    QAfterFilterCondition
  >
  idBetween(
    Id lower,
    Id upper, {
    bool includeLower = true,
    bool includeUpper = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.between(
          property: r'id',
          lower: lower,
          includeLower: includeLower,
          upper: upper,
          includeUpper: includeUpper,
        ),
      );
    });
  }

  QueryBuilder<
    WallpaperAssignmentRecord,
    WallpaperAssignmentRecord,
    QAfterFilterCondition
  >
  outputEqualTo(String value, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.equalTo(
          property: r'output',
          value: value,
          caseSensitive: caseSensitive,
        ),
      );
    });
  }

  QueryBuilder<
    WallpaperAssignmentRecord,
    WallpaperAssignmentRecord,
    QAfterFilterCondition
  >
  outputGreaterThan(
    String value, {
    bool include = false,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.greaterThan(
          include: include,
          property: r'output',
          value: value,
          caseSensitive: caseSensitive,
        ),
      );
    });
  }

  QueryBuilder<
    WallpaperAssignmentRecord,
    WallpaperAssignmentRecord,
    QAfterFilterCondition
  >
  outputLessThan(
    String value, {
    bool include = false,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.lessThan(
          include: include,
          property: r'output',
          value: value,
          caseSensitive: caseSensitive,
        ),
      );
    });
  }

  QueryBuilder<
    WallpaperAssignmentRecord,
    WallpaperAssignmentRecord,
    QAfterFilterCondition
  >
  outputBetween(
    String lower,
    String upper, {
    bool includeLower = true,
    bool includeUpper = true,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.between(
          property: r'output',
          lower: lower,
          includeLower: includeLower,
          upper: upper,
          includeUpper: includeUpper,
          caseSensitive: caseSensitive,
        ),
      );
    });
  }

  QueryBuilder<
    WallpaperAssignmentRecord,
    WallpaperAssignmentRecord,
    QAfterFilterCondition
  >
  outputStartsWith(String value, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.startsWith(
          property: r'output',
          value: value,
          caseSensitive: caseSensitive,
        ),
      );
    });
  }

  QueryBuilder<
    WallpaperAssignmentRecord,
    WallpaperAssignmentRecord,
    QAfterFilterCondition
  >
  outputEndsWith(String value, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.endsWith(
          property: r'output',
          value: value,
          caseSensitive: caseSensitive,
        ),
      );
    });
  }

  QueryBuilder<
    WallpaperAssignmentRecord,
    WallpaperAssignmentRecord,
    QAfterFilterCondition
  >
  outputContains(String value, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.contains(
          property: r'output',
          value: value,
          caseSensitive: caseSensitive,
        ),
      );
    });
  }

  QueryBuilder<
    WallpaperAssignmentRecord,
    WallpaperAssignmentRecord,
    QAfterFilterCondition
  >
  outputMatches(String pattern, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.matches(
          property: r'output',
          wildcard: pattern,
          caseSensitive: caseSensitive,
        ),
      );
    });
  }

  QueryBuilder<
    WallpaperAssignmentRecord,
    WallpaperAssignmentRecord,
    QAfterFilterCondition
  >
  outputIsEmpty() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.equalTo(property: r'output', value: ''),
      );
    });
  }

  QueryBuilder<
    WallpaperAssignmentRecord,
    WallpaperAssignmentRecord,
    QAfterFilterCondition
  >
  outputIsNotEmpty() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.greaterThan(property: r'output', value: ''),
      );
    });
  }

  QueryBuilder<
    WallpaperAssignmentRecord,
    WallpaperAssignmentRecord,
    QAfterFilterCondition
  >
  sourceIdIsNull() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        const FilterCondition.isNull(property: r'sourceId'),
      );
    });
  }

  QueryBuilder<
    WallpaperAssignmentRecord,
    WallpaperAssignmentRecord,
    QAfterFilterCondition
  >
  sourceIdIsNotNull() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        const FilterCondition.isNotNull(property: r'sourceId'),
      );
    });
  }

  QueryBuilder<
    WallpaperAssignmentRecord,
    WallpaperAssignmentRecord,
    QAfterFilterCondition
  >
  sourceIdEqualTo(String? value, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.equalTo(
          property: r'sourceId',
          value: value,
          caseSensitive: caseSensitive,
        ),
      );
    });
  }

  QueryBuilder<
    WallpaperAssignmentRecord,
    WallpaperAssignmentRecord,
    QAfterFilterCondition
  >
  sourceIdGreaterThan(
    String? value, {
    bool include = false,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.greaterThan(
          include: include,
          property: r'sourceId',
          value: value,
          caseSensitive: caseSensitive,
        ),
      );
    });
  }

  QueryBuilder<
    WallpaperAssignmentRecord,
    WallpaperAssignmentRecord,
    QAfterFilterCondition
  >
  sourceIdLessThan(
    String? value, {
    bool include = false,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.lessThan(
          include: include,
          property: r'sourceId',
          value: value,
          caseSensitive: caseSensitive,
        ),
      );
    });
  }

  QueryBuilder<
    WallpaperAssignmentRecord,
    WallpaperAssignmentRecord,
    QAfterFilterCondition
  >
  sourceIdBetween(
    String? lower,
    String? upper, {
    bool includeLower = true,
    bool includeUpper = true,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.between(
          property: r'sourceId',
          lower: lower,
          includeLower: includeLower,
          upper: upper,
          includeUpper: includeUpper,
          caseSensitive: caseSensitive,
        ),
      );
    });
  }

  QueryBuilder<
    WallpaperAssignmentRecord,
    WallpaperAssignmentRecord,
    QAfterFilterCondition
  >
  sourceIdStartsWith(String value, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.startsWith(
          property: r'sourceId',
          value: value,
          caseSensitive: caseSensitive,
        ),
      );
    });
  }

  QueryBuilder<
    WallpaperAssignmentRecord,
    WallpaperAssignmentRecord,
    QAfterFilterCondition
  >
  sourceIdEndsWith(String value, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.endsWith(
          property: r'sourceId',
          value: value,
          caseSensitive: caseSensitive,
        ),
      );
    });
  }

  QueryBuilder<
    WallpaperAssignmentRecord,
    WallpaperAssignmentRecord,
    QAfterFilterCondition
  >
  sourceIdContains(String value, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.contains(
          property: r'sourceId',
          value: value,
          caseSensitive: caseSensitive,
        ),
      );
    });
  }

  QueryBuilder<
    WallpaperAssignmentRecord,
    WallpaperAssignmentRecord,
    QAfterFilterCondition
  >
  sourceIdMatches(String pattern, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.matches(
          property: r'sourceId',
          wildcard: pattern,
          caseSensitive: caseSensitive,
        ),
      );
    });
  }

  QueryBuilder<
    WallpaperAssignmentRecord,
    WallpaperAssignmentRecord,
    QAfterFilterCondition
  >
  sourceIdIsEmpty() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.equalTo(property: r'sourceId', value: ''),
      );
    });
  }

  QueryBuilder<
    WallpaperAssignmentRecord,
    WallpaperAssignmentRecord,
    QAfterFilterCondition
  >
  sourceIdIsNotEmpty() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.greaterThan(property: r'sourceId', value: ''),
      );
    });
  }

  QueryBuilder<
    WallpaperAssignmentRecord,
    WallpaperAssignmentRecord,
    QAfterFilterCondition
  >
  sourcePathEqualTo(String value, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.equalTo(
          property: r'sourcePath',
          value: value,
          caseSensitive: caseSensitive,
        ),
      );
    });
  }

  QueryBuilder<
    WallpaperAssignmentRecord,
    WallpaperAssignmentRecord,
    QAfterFilterCondition
  >
  sourcePathGreaterThan(
    String value, {
    bool include = false,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.greaterThan(
          include: include,
          property: r'sourcePath',
          value: value,
          caseSensitive: caseSensitive,
        ),
      );
    });
  }

  QueryBuilder<
    WallpaperAssignmentRecord,
    WallpaperAssignmentRecord,
    QAfterFilterCondition
  >
  sourcePathLessThan(
    String value, {
    bool include = false,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.lessThan(
          include: include,
          property: r'sourcePath',
          value: value,
          caseSensitive: caseSensitive,
        ),
      );
    });
  }

  QueryBuilder<
    WallpaperAssignmentRecord,
    WallpaperAssignmentRecord,
    QAfterFilterCondition
  >
  sourcePathBetween(
    String lower,
    String upper, {
    bool includeLower = true,
    bool includeUpper = true,
    bool caseSensitive = true,
  }) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.between(
          property: r'sourcePath',
          lower: lower,
          includeLower: includeLower,
          upper: upper,
          includeUpper: includeUpper,
          caseSensitive: caseSensitive,
        ),
      );
    });
  }

  QueryBuilder<
    WallpaperAssignmentRecord,
    WallpaperAssignmentRecord,
    QAfterFilterCondition
  >
  sourcePathStartsWith(String value, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.startsWith(
          property: r'sourcePath',
          value: value,
          caseSensitive: caseSensitive,
        ),
      );
    });
  }

  QueryBuilder<
    WallpaperAssignmentRecord,
    WallpaperAssignmentRecord,
    QAfterFilterCondition
  >
  sourcePathEndsWith(String value, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.endsWith(
          property: r'sourcePath',
          value: value,
          caseSensitive: caseSensitive,
        ),
      );
    });
  }

  QueryBuilder<
    WallpaperAssignmentRecord,
    WallpaperAssignmentRecord,
    QAfterFilterCondition
  >
  sourcePathContains(String value, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.contains(
          property: r'sourcePath',
          value: value,
          caseSensitive: caseSensitive,
        ),
      );
    });
  }

  QueryBuilder<
    WallpaperAssignmentRecord,
    WallpaperAssignmentRecord,
    QAfterFilterCondition
  >
  sourcePathMatches(String pattern, {bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.matches(
          property: r'sourcePath',
          wildcard: pattern,
          caseSensitive: caseSensitive,
        ),
      );
    });
  }

  QueryBuilder<
    WallpaperAssignmentRecord,
    WallpaperAssignmentRecord,
    QAfterFilterCondition
  >
  sourcePathIsEmpty() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.equalTo(property: r'sourcePath', value: ''),
      );
    });
  }

  QueryBuilder<
    WallpaperAssignmentRecord,
    WallpaperAssignmentRecord,
    QAfterFilterCondition
  >
  sourcePathIsNotEmpty() {
    return QueryBuilder.apply(this, (query) {
      return query.addFilterCondition(
        FilterCondition.greaterThan(property: r'sourcePath', value: ''),
      );
    });
  }
}

extension WallpaperAssignmentRecordQueryObject
    on
        QueryBuilder<
          WallpaperAssignmentRecord,
          WallpaperAssignmentRecord,
          QFilterCondition
        > {}

extension WallpaperAssignmentRecordQueryLinks
    on
        QueryBuilder<
          WallpaperAssignmentRecord,
          WallpaperAssignmentRecord,
          QFilterCondition
        > {}

extension WallpaperAssignmentRecordQuerySortBy
    on
        QueryBuilder<
          WallpaperAssignmentRecord,
          WallpaperAssignmentRecord,
          QSortBy
        > {
  QueryBuilder<
    WallpaperAssignmentRecord,
    WallpaperAssignmentRecord,
    QAfterSortBy
  >
  sortByOutput() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'output', Sort.asc);
    });
  }

  QueryBuilder<
    WallpaperAssignmentRecord,
    WallpaperAssignmentRecord,
    QAfterSortBy
  >
  sortByOutputDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'output', Sort.desc);
    });
  }

  QueryBuilder<
    WallpaperAssignmentRecord,
    WallpaperAssignmentRecord,
    QAfterSortBy
  >
  sortBySourceId() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'sourceId', Sort.asc);
    });
  }

  QueryBuilder<
    WallpaperAssignmentRecord,
    WallpaperAssignmentRecord,
    QAfterSortBy
  >
  sortBySourceIdDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'sourceId', Sort.desc);
    });
  }

  QueryBuilder<
    WallpaperAssignmentRecord,
    WallpaperAssignmentRecord,
    QAfterSortBy
  >
  sortBySourcePath() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'sourcePath', Sort.asc);
    });
  }

  QueryBuilder<
    WallpaperAssignmentRecord,
    WallpaperAssignmentRecord,
    QAfterSortBy
  >
  sortBySourcePathDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'sourcePath', Sort.desc);
    });
  }
}

extension WallpaperAssignmentRecordQuerySortThenBy
    on
        QueryBuilder<
          WallpaperAssignmentRecord,
          WallpaperAssignmentRecord,
          QSortThenBy
        > {
  QueryBuilder<
    WallpaperAssignmentRecord,
    WallpaperAssignmentRecord,
    QAfterSortBy
  >
  thenById() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'id', Sort.asc);
    });
  }

  QueryBuilder<
    WallpaperAssignmentRecord,
    WallpaperAssignmentRecord,
    QAfterSortBy
  >
  thenByIdDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'id', Sort.desc);
    });
  }

  QueryBuilder<
    WallpaperAssignmentRecord,
    WallpaperAssignmentRecord,
    QAfterSortBy
  >
  thenByOutput() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'output', Sort.asc);
    });
  }

  QueryBuilder<
    WallpaperAssignmentRecord,
    WallpaperAssignmentRecord,
    QAfterSortBy
  >
  thenByOutputDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'output', Sort.desc);
    });
  }

  QueryBuilder<
    WallpaperAssignmentRecord,
    WallpaperAssignmentRecord,
    QAfterSortBy
  >
  thenBySourceId() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'sourceId', Sort.asc);
    });
  }

  QueryBuilder<
    WallpaperAssignmentRecord,
    WallpaperAssignmentRecord,
    QAfterSortBy
  >
  thenBySourceIdDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'sourceId', Sort.desc);
    });
  }

  QueryBuilder<
    WallpaperAssignmentRecord,
    WallpaperAssignmentRecord,
    QAfterSortBy
  >
  thenBySourcePath() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'sourcePath', Sort.asc);
    });
  }

  QueryBuilder<
    WallpaperAssignmentRecord,
    WallpaperAssignmentRecord,
    QAfterSortBy
  >
  thenBySourcePathDesc() {
    return QueryBuilder.apply(this, (query) {
      return query.addSortBy(r'sourcePath', Sort.desc);
    });
  }
}

extension WallpaperAssignmentRecordQueryWhereDistinct
    on
        QueryBuilder<
          WallpaperAssignmentRecord,
          WallpaperAssignmentRecord,
          QDistinct
        > {
  QueryBuilder<WallpaperAssignmentRecord, WallpaperAssignmentRecord, QDistinct>
  distinctByOutput({bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addDistinctBy(r'output', caseSensitive: caseSensitive);
    });
  }

  QueryBuilder<WallpaperAssignmentRecord, WallpaperAssignmentRecord, QDistinct>
  distinctBySourceId({bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addDistinctBy(r'sourceId', caseSensitive: caseSensitive);
    });
  }

  QueryBuilder<WallpaperAssignmentRecord, WallpaperAssignmentRecord, QDistinct>
  distinctBySourcePath({bool caseSensitive = true}) {
    return QueryBuilder.apply(this, (query) {
      return query.addDistinctBy(r'sourcePath', caseSensitive: caseSensitive);
    });
  }
}

extension WallpaperAssignmentRecordQueryProperty
    on
        QueryBuilder<
          WallpaperAssignmentRecord,
          WallpaperAssignmentRecord,
          QQueryProperty
        > {
  QueryBuilder<WallpaperAssignmentRecord, int, QQueryOperations> idProperty() {
    return QueryBuilder.apply(this, (query) {
      return query.addPropertyName(r'id');
    });
  }

  QueryBuilder<WallpaperAssignmentRecord, String, QQueryOperations>
  outputProperty() {
    return QueryBuilder.apply(this, (query) {
      return query.addPropertyName(r'output');
    });
  }

  QueryBuilder<WallpaperAssignmentRecord, String?, QQueryOperations>
  sourceIdProperty() {
    return QueryBuilder.apply(this, (query) {
      return query.addPropertyName(r'sourceId');
    });
  }

  QueryBuilder<WallpaperAssignmentRecord, String, QQueryOperations>
  sourcePathProperty() {
    return QueryBuilder.apply(this, (query) {
      return query.addPropertyName(r'sourcePath');
    });
  }
}
